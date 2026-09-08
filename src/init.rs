use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{HnmError, Result};
use crate::plan::{FileKind, PlanEntry, harness_plan};
use crate::render::{self, TemplateContext};
use crate::stack::Stack;

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub target: PathBuf,
    pub project_name: String,
    pub stack: Stack,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Create,
    Overwrite,
    SkipExists,
    Link,
    Relink,
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
                    ActionKind::Overwrite => "overwrite",
                    ActionKind::SkipExists => "skip",
                    ActionKind::Link => "link",
                    ActionKind::Relink => "relink",
                    ActionKind::SkipLinkOk => "skip-link",
                };
                format!("{label:10} {}", a.rel)
            })
            .collect()
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
            PlanEntry::File { rel, kind } => {
                let content = match kind {
                    FileKind::AgentsTemplate => render::render_agents(&ctx)?,
                    FileKind::SpecTemplate => render::render_spec(&ctx)?,
                    FileKind::Static(body) => body.to_string(),
                };
                let report = write_file(&opts.target, rel, &content, opts.force, opts.dry_run)?;
                actions.push(report);
            }
            PlanEntry::Symlink { rel, target } => {
                let report = write_symlink(&opts.target, rel, target, opts.force, opts.dry_run)?;
                actions.push(report);
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

fn write_file(
    root: &Path,
    rel: &str,
    content: &str,
    force: bool,
    dry_run: bool,
) -> Result<ActionReport> {
    let path = root.join(rel);
    let exists = path.exists() || path.symlink_metadata().is_ok();

    if exists && !force {
        return Ok(ActionReport {
            rel: rel.to_string(),
            kind: ActionKind::SkipExists,
        });
    }

    let kind = if exists {
        ActionKind::Overwrite
    } else {
        ActionKind::Create
    };

    if dry_run {
        return Ok(ActionReport {
            rel: rel.to_string(),
            kind,
        });
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| HnmError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    if exists {
        // Remove symlink or file so open/write is clean.
        fs::remove_file(&path)
            .or_else(|_| fs::remove_dir_all(&path))
            .map_err(|source| HnmError::Remove {
                path: path.clone(),
                source,
            })?;
    }

    let mut file = fs::File::create(&path).map_err(|source| HnmError::WriteFile {
        path: path.clone(),
        source,
    })?;
    file.write_all(content.as_bytes())
        .map_err(|source| HnmError::WriteFile {
            path: path.clone(),
            source,
        })?;
    // Ensure trailing newline for text assets that forgot one.
    if !content.is_empty() && !content.ends_with('\n') {
        file.write_all(b"\n")
            .map_err(|source| HnmError::WriteFile { path, source })?;
    }

    Ok(ActionReport {
        rel: rel.to_string(),
        kind,
    })
}

fn write_symlink(
    root: &Path,
    rel: &str,
    target: &str,
    force: bool,
    dry_run: bool,
) -> Result<ActionReport> {
    let link_path = root.join(rel);
    let meta = fs::symlink_metadata(&link_path);

    if let Ok(meta) = meta {
        if meta.file_type().is_symlink()
            && fs::read_link(&link_path).is_ok_and(|existing| existing == Path::new(target))
        {
            return Ok(ActionReport {
                rel: rel.to_string(),
                kind: ActionKind::SkipLinkOk,
            });
        }
        if !force {
            return Ok(ActionReport {
                rel: rel.to_string(),
                kind: ActionKind::SkipExists,
            });
        }
    }

    let exists = link_path.exists() || fs::symlink_metadata(&link_path).is_ok();
    let kind = if exists {
        ActionKind::Relink
    } else {
        ActionKind::Link
    };

    if dry_run {
        return Ok(ActionReport {
            rel: rel.to_string(),
            kind,
        });
    }

    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent).map_err(|source| HnmError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    if exists {
        if link_path.is_dir() && !link_path.is_symlink() {
            fs::remove_dir_all(&link_path).map_err(|source| HnmError::Remove {
                path: link_path.clone(),
                source,
            })?;
        } else {
            fs::remove_file(&link_path).map_err(|source| HnmError::Remove {
                path: link_path.clone(),
                source,
            })?;
        }
    }

    create_symlink(Path::new(target), &link_path)?;

    Ok(ActionReport {
        rel: rel.to_string(),
        kind,
    })
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
            force: false,
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
            force: false,
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
            force: false,
            dry_run: true,
        };
        run_init(&opts).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn skip_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "keep-me\n").unwrap();

        let opts = InitOptions {
            target: dir.path().to_path_buf(),
            project_name: "x".into(),
            stack: Stack::Generic,
            force: false,
            dry_run: false,
        };
        let report = run_init(&opts).unwrap();
        let agents = report
            .actions
            .iter()
            .find(|a| a.rel == "AGENTS.md")
            .unwrap();
        assert_eq!(agents.kind, ActionKind::SkipExists);
        assert_eq!(fs::read_to_string(path).unwrap(), "keep-me\n");
    }

    #[test]
    fn force_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "old\n").unwrap();

        let opts = InitOptions {
            target: dir.path().to_path_buf(),
            project_name: "forced".into(),
            stack: Stack::Bun,
            force: true,
            dry_run: false,
        };
        run_init(&opts).unwrap();
        let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
        assert!(agents.contains("`forced`"));
        assert!(agents.contains("bun test"));
        assert!(!agents.starts_with("old"));
    }
}
