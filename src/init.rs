use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{HnmError, Result};
use crate::merge::{self, Merged};
use crate::plan::{FileKind, LinkFallback, PlanEntry, SKILLS, Update, harness_plan};
use crate::render::{self, TemplateContext};
use crate::stack::Stack;

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub target: PathBuf,
    pub project_name: String,
    pub stack: Stack,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Create,
    /// Generated content merged into an existing file.
    Update,
    /// Existing file already carries the generated content.
    Unchanged,
    /// Existing content could not be merged; left untouched.
    Conflict,
    Link,
    SkipLinkOk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionReport {
    pub rel: String,
    pub kind: ActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub actions: Vec<ActionReport>,
}

impl InitReport {
    pub fn summary_lines(&self) -> Vec<String> {
        self.actions
            .iter()
            .map(|a| {
                let label = match a.kind {
                    ActionKind::Create => "create",
                    ActionKind::Update => "update",
                    ActionKind::Unchanged => "unchanged",
                    ActionKind::Conflict => "skip",
                    ActionKind::Link => "link",
                    ActionKind::SkipLinkOk => "skip-link",
                };
                format!("{label:10} {}", a.rel)
            })
            .collect()
    }

    pub fn conflicts(&self) -> usize {
        self.actions
            .iter()
            .filter(|a| a.kind == ActionKind::Conflict)
            .count()
    }
}

pub fn resolve_project_name(explicit: Option<String>, target: &Path) -> String {
    if let Some(name) = explicit {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    target
        .file_name()
        .and_then(|s| s.to_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .unwrap_or("project")
        .to_string()
}

pub fn run_init(opts: &InitOptions) -> Result<InitReport> {
    if !opts.dry_run || opts.target.exists() {
        ensure_target_dir(&opts.target)?;
    }

    let ctx = TemplateContext {
        project_name: &opts.project_name,
        stack: opts.stack,
    };

    let mut actions = Vec::new();
    for entry in harness_plan() {
        match entry {
            PlanEntry::File { rel, kind, update } => {
                let content = match kind {
                    FileKind::AgentsTemplate => render::render_agents(&ctx)?,
                    FileKind::SpecTemplate => render::render_spec(&ctx)?,
                    FileKind::Static(body) => body.to_string(),
                };
                actions.push(write_file(
                    &opts.target,
                    rel,
                    &content,
                    update,
                    opts.dry_run,
                )?);
            }
            PlanEntry::Symlink {
                rel,
                target,
                fallback,
            } => {
                actions.extend(write_symlink(
                    &opts.target,
                    rel,
                    target,
                    fallback,
                    opts.dry_run,
                )?);
            }
        }
    }

    Ok(InitReport { actions })
}

fn ensure_target_dir(target: &Path) -> Result<()> {
    if target.exists() {
        if target.is_dir() {
            return Ok(());
        }
        return Err(HnmError::NotADirectory(target.to_path_buf()));
    }
    fs::create_dir_all(target).map_err(|source| HnmError::CreateDir {
        path: target.to_path_buf(),
        source,
    })
}

fn report(rel: &str, kind: ActionKind) -> ActionReport {
    ActionReport {
        rel: rel.to_string(),
        kind,
    }
}

fn write_file(
    root: &Path,
    rel: &str,
    content: &str,
    update: Update,
    dry_run: bool,
) -> Result<ActionReport> {
    let path = root.join(rel);
    let content = with_trailing_newline(content);

    if fs::symlink_metadata(&path).is_err() {
        if !dry_run {
            write_contents(&path, &content)?;
        }
        return Ok(report(rel, ActionKind::Create));
    }

    // Unreadable paths (directories, non-UTF-8, dangling links) are conflicts.
    let merged = match fs::read_to_string(&path) {
        Ok(existing) if existing.trim_end() == content.trim_end() => Merged::Unchanged,
        Ok(existing) => match update {
            Update::Keep => Merged::Conflict,
            Update::ManagedBlock => merge::upsert_block(&existing, &content),
            Update::JsonMerge => merge::merge_json(&existing, &content),
        },
        Err(_) => Merged::Conflict,
    };
    apply_merge(&path, rel, merged, dry_run)
}

fn apply_merge(path: &Path, rel: &str, merged: Merged, dry_run: bool) -> Result<ActionReport> {
    let kind = match merged {
        Merged::Unchanged => ActionKind::Unchanged,
        Merged::Conflict => ActionKind::Conflict,
        Merged::Updated(next) => {
            if !dry_run {
                write_contents(path, &next)?;
            }
            ActionKind::Update
        }
    };
    Ok(report(rel, kind))
}

fn with_trailing_newline(content: &str) -> String {
    let mut out = content.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn write_contents(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| HnmError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, content).map_err(|source| HnmError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn write_symlink(
    root: &Path,
    rel: &str,
    target: &str,
    fallback: LinkFallback,
    dry_run: bool,
) -> Result<Vec<ActionReport>> {
    let link_path = root.join(rel);
    let Ok(meta) = fs::symlink_metadata(&link_path) else {
        if !dry_run {
            if let Some(parent) = link_path.parent() {
                fs::create_dir_all(parent).map_err(|source| HnmError::CreateDir {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            create_symlink(Path::new(target), &link_path)?;
        }
        return Ok(vec![report(rel, ActionKind::Link)]);
    };

    if meta.file_type().is_symlink() {
        let kind = if fs::read_link(&link_path).is_ok_and(|existing| existing == Path::new(target))
        {
            ActionKind::SkipLinkOk
        } else {
            ActionKind::Conflict
        };
        return Ok(vec![report(rel, kind)]);
    }

    match fallback {
        LinkFallback::ImportBlock if meta.is_file() => {
            let resolved = link_path.parent().unwrap_or(root).join(target);
            // A file that is the link target itself already has the content.
            if same_file(&link_path, &resolved) {
                return Ok(vec![report(rel, ActionKind::Unchanged)]);
            }
            let merged = match fs::read_to_string(&link_path) {
                Ok(existing) => merge::ensure_import(&existing, target),
                Err(_) => Merged::Conflict,
            };
            Ok(vec![apply_merge(&link_path, rel, merged, dry_run)?])
        }
        LinkFallback::LinkSkills if meta.is_dir() => {
            let mut reports = Vec::new();
            for skill in SKILLS {
                reports.extend(write_symlink(
                    root,
                    &format!("{rel}/{skill}"),
                    &format!("../{target}/{skill}"),
                    LinkFallback::None,
                    dry_run,
                )?);
            }
            Ok(reports)
        }
        _ => Ok(vec![report(rel, ActionKind::Conflict)]),
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).map_err(|source| HnmError::Symlink {
        link: link.to_path_buf(),
        target: target.to_path_buf(),
        source,
    })
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    // Harness links point at files (CLAUDE.md) or directories (.claude/skills).
    let result = if target.extension().is_some() || target == Path::new("AGENTS.md") {
        std::os::windows::fs::symlink_file(target, link)
    } else {
        std::os::windows::fs::symlink_dir(target, link)
    };
    result.map_err(|source| HnmError::Symlink {
        link: link.to_path_buf(),
        target: target.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_name_prefers_explicit() {
        assert_eq!(
            resolve_project_name(Some("  demo  ".into()), Path::new("/tmp/other")),
            "demo"
        );
    }

    #[test]
    fn resolve_name_from_dir() {
        assert_eq!(
            resolve_project_name(None, Path::new("/tmp/my-app")),
            "my-app"
        );
    }

    #[test]
    fn init_writes_full_harness() {
        let dir = tempfile::tempdir().unwrap();
        let opts = InitOptions {
            target: dir.path().to_path_buf(),
            project_name: "sample".into(),
            stack: Stack::Rust,
            dry_run: false,
        };
        let report = run_init(&opts).unwrap();
        assert!(
            report
                .actions
                .iter()
                .all(|a| matches!(a.kind, ActionKind::Create | ActionKind::Link))
        );

        let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
        assert!(agents.contains("`sample`"));
        assert!(agents.contains("cargo test"));

        let skill = dir.path().join(".agents/skills/feature-dev/SKILL.md");
        assert!(skill.is_file());
        assert!(fs::read_to_string(&skill).unwrap().contains("质问"));

        let git_commit = dir.path().join(".agents/skills/git-commit/SKILL.md");
        assert!(git_commit.is_file());
        assert!(
            fs::read_to_string(&git_commit)
                .unwrap()
                .contains("Conventional Commits")
        );
        assert!(
            dir.path()
                .join(".agents/skills/git-commit/agents/openai.yaml")
                .is_file()
        );

        let claude = dir.path().join("CLAUDE.md");
        assert!(claude.is_symlink());
        assert_eq!(fs::read_link(&claude).unwrap(), Path::new("AGENTS.md"));

        let skills_link = dir.path().join(".claude/skills");
        assert!(skills_link.is_symlink());
        assert_eq!(
            fs::read_link(&skills_link).unwrap(),
            Path::new("../.agents/skills")
        );

        assert!(dir.path().join(".codex/hooks/spec_doc_review.py").is_file());
        assert!(dir.path().join("docs/prd/.gitkeep").is_file());
        assert!(dir.path().join("docs/adr/.gitkeep").is_file());
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let opts = InitOptions {
            target: dir.path().to_path_buf(),
            project_name: "x".into(),
            stack: Stack::Generic,
            dry_run: true,
        };
        run_init(&opts).unwrap();
        assert!(!dir.path().join("AGENTS.md").exists());
    }

    #[test]
    fn dry_run_does_not_create_missing_target() {
        let dir = tempfile::tempdir().unwrap();
        let opts = InitOptions {
            target: dir.path().join("missing/nested"),
            project_name: "x".into(),
            stack: Stack::Generic,
            dry_run: true,
        };
        run_init(&opts).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    fn opts(dir: &Path) -> InitOptions {
        InitOptions {
            target: dir.to_path_buf(),
            project_name: "merged".into(),
            stack: Stack::Rust,
            dry_run: false,
        }
    }

    fn kind_of(report: &InitReport, rel: &str) -> ActionKind {
        report.actions.iter().find(|a| a.rel == rel).unwrap().kind
    }

    #[test]
    fn existing_agents_gets_managed_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "# Keep me\n").unwrap();

        let report = run_init(&opts(dir.path())).unwrap();
        assert_eq!(kind_of(&report, "AGENTS.md"), ActionKind::Update);
        let agents = fs::read_to_string(&path).unwrap();
        assert!(agents.starts_with("# Keep me\n"));
        assert!(agents.contains(merge::BLOCK_BEGIN));
        assert!(agents.contains("`merged`"));

        let again = run_init(&opts(dir.path())).unwrap();
        assert_eq!(kind_of(&again, "AGENTS.md"), ActionKind::Unchanged);
        assert_eq!(fs::read_to_string(&path).unwrap(), agents);
    }

    #[test]
    fn regular_claude_md_imports_agents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "my notes\n").unwrap();

        let report = run_init(&opts(dir.path())).unwrap();
        assert_eq!(kind_of(&report, "CLAUDE.md"), ActionKind::Update);
        let claude = fs::read_to_string(&path).unwrap();
        assert!(claude.starts_with("my notes\n"));
        assert!(claude.contains("\n@AGENTS.md\n"));
    }

    #[test]
    fn existing_settings_json_is_merged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude/settings.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"model":"opus"}"#).unwrap();

        let report = run_init(&opts(dir.path())).unwrap();
        assert_eq!(
            kind_of(&report, ".claude/settings.json"),
            ActionKind::Update
        );
        let json = fs::read_to_string(&path).unwrap();
        assert!(json.contains("\"model\": \"opus\""));
        assert!(json.contains("spec_doc_review.py"));
    }

    #[test]
    fn real_skills_dir_gets_per_skill_links() {
        let dir = tempfile::tempdir().unwrap();
        let skills = dir.path().join(".claude/skills");
        fs::create_dir_all(skills.join("mine")).unwrap();

        let report = run_init(&opts(dir.path())).unwrap();
        assert!(skills.join("mine").is_dir());
        for skill in SKILLS {
            let link = skills.join(skill);
            assert_eq!(
                kind_of(&report, &format!(".claude/skills/{skill}")),
                ActionKind::Link
            );
            assert!(link.join("SKILL.md").is_file());
        }
    }

    #[test]
    fn unmergeable_file_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let spec = dir.path().join("docs/spec.md");
        fs::create_dir_all(spec.parent().unwrap()).unwrap();
        fs::write(&spec, "my spec\n").unwrap();
        let settings = dir.path().join(".claude/settings.json");
        fs::create_dir_all(settings.parent().unwrap()).unwrap();
        fs::write(&settings, "{ broken").unwrap();

        let report = run_init(&opts(dir.path())).unwrap();
        assert_eq!(kind_of(&report, "docs/spec.md"), ActionKind::Conflict);
        assert_eq!(
            kind_of(&report, ".claude/settings.json"),
            ActionKind::Conflict
        );
        assert_eq!(report.conflicts(), 2);
        assert_eq!(fs::read_to_string(spec).unwrap(), "my spec\n");
        assert_eq!(fs::read_to_string(settings).unwrap(), "{ broken");
    }

    #[test]
    fn rerun_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        run_init(&opts(dir.path())).unwrap();
        let report = run_init(&opts(dir.path())).unwrap();
        assert!(
            report
                .actions
                .iter()
                .all(|a| matches!(a.kind, ActionKind::Unchanged | ActionKind::SkipLinkOk))
        );
    }

    #[test]
    fn dry_run_merge_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "# Keep me\n").unwrap();
        let mut o = opts(dir.path());
        o.dry_run = true;
        let report = run_init(&o).unwrap();
        assert_eq!(kind_of(&report, "AGENTS.md"), ActionKind::Update);
        assert_eq!(fs::read_to_string(path).unwrap(), "# Keep me\n");
    }
}
