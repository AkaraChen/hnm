/// One filesystem action in the harness install plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanEntry {
    /// Write UTF-8 file content (rendered or static).
    File {
        /// Path relative to the target root, using `/` separators.
        rel: &'static str,
        kind: FileKind,
        /// How to fold generated content into a file the user already has.
        update: Update,
    },
    /// Create a relative symlink `rel` -> `target`.
    Symlink {
        rel: &'static str,
        target: &'static str,
        /// What to do when something else already occupies `rel`.
        fallback: LinkFallback,
    },
}

/// Best-effort strategy for a file that already exists at a plan path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Update {
    /// Leave the user's file alone.
    Keep,
    /// Insert or refresh an hnm-managed Markdown block.
    ManagedBlock,
    /// Deep-merge JSON objects and append missing array items.
    JsonMerge,
}

/// Best-effort strategy when a symlink path is already occupied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkFallback {
    None,
    /// Regular file: add a managed block importing the link target (`@AGENTS.md`).
    ImportBlock,
    /// Real directory: link each harness skill inside it instead.
    LinkSkills,
}

/// Skills installed under `.agents/skills/`.
pub const SKILLS: &[&str] = &["feature-dev", "git-commit"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    AgentsTemplate,
    SpecTemplate,
    Static(&'static str),
}

/// Complete ordered harness layout. Keep in sync with docs/spec.md.
pub fn harness_plan() -> Vec<PlanEntry> {
    vec![
        PlanEntry::File {
            rel: "AGENTS.md",
            kind: FileKind::AgentsTemplate,
            update: Update::ManagedBlock,
        },
        PlanEntry::Symlink {
            rel: "CLAUDE.md",
            target: "AGENTS.md",
            fallback: LinkFallback::ImportBlock,
        },
        PlanEntry::File {
            rel: "docs/spec.md",
            kind: FileKind::SpecTemplate,
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: "docs/prd/.gitkeep",
            kind: FileKind::Static(include_str!("../templates/docs/prd/.gitkeep")),
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: "docs/adr/.gitkeep",
            kind: FileKind::Static(include_str!("../templates/docs/adr/.gitkeep")),
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: ".agents/skills/feature-dev/SKILL.md",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/feature-dev/SKILL.md"
            )),
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: ".agents/skills/feature-dev/agents/openai.yaml",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/feature-dev/agents/openai.yaml"
            )),
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: ".agents/skills/git-commit/SKILL.md",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/git-commit/SKILL.md"
            )),
            update: Update::Keep,
        },
        PlanEntry::File {
            rel: ".agents/skills/git-commit/agents/openai.yaml",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/git-commit/agents/openai.yaml"
            )),
            update: Update::Keep,
        },
        PlanEntry::Symlink {
            rel: ".claude/skills",
            target: "../.agents/skills",
            fallback: LinkFallback::LinkSkills,
        },
        PlanEntry::File {
            rel: ".claude/settings.json",
            kind: FileKind::Static(include_str!("../templates/.claude/settings.json")),
            update: Update::JsonMerge,
        },
        PlanEntry::File {
            rel: ".codex/hooks.json",
            kind: FileKind::Static(include_str!("../templates/.codex/hooks.json")),
            update: Update::JsonMerge,
        },
        PlanEntry::File {
            rel: ".codex/hooks/spec_doc_review.py",
            kind: FileKind::Static(include_str!("../templates/.codex/hooks/spec_doc_review.py")),
            update: Update::Keep,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn plan_covers_required_paths() {
        let rels: HashSet<&str> = harness_plan()
            .into_iter()
            .map(|e| match e {
                PlanEntry::File { rel, .. } | PlanEntry::Symlink { rel, .. } => rel,
            })
            .collect();
        for required in [
            "AGENTS.md",
            "CLAUDE.md",
            "docs/spec.md",
            "docs/prd/.gitkeep",
            "docs/adr/.gitkeep",
            ".agents/skills/feature-dev/SKILL.md",
            ".agents/skills/feature-dev/agents/openai.yaml",
            ".agents/skills/git-commit/SKILL.md",
            ".agents/skills/git-commit/agents/openai.yaml",
            ".claude/skills",
            ".claude/settings.json",
            ".codex/hooks.json",
            ".codex/hooks/spec_doc_review.py",
        ] {
            assert!(rels.contains(required), "missing {required}");
        }
    }
}
