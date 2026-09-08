/// One filesystem action in the harness install plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanEntry {
    /// Write UTF-8 file content (rendered or static).
    File {
        /// Path relative to the target root, using `/` separators.
        rel: &'static str,
        kind: FileKind,
    },
    /// Create a relative symlink `rel` -> `target`.
    Symlink {
        rel: &'static str,
        target: &'static str,
    },
}

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
        },
        PlanEntry::Symlink {
            rel: "CLAUDE.md",
            target: "AGENTS.md",
        },
        PlanEntry::File {
            rel: "docs/spec.md",
            kind: FileKind::SpecTemplate,
        },
        PlanEntry::File {
            rel: "docs/prd/.gitkeep",
            kind: FileKind::Static(include_str!("../templates/docs/prd/.gitkeep")),
        },
        PlanEntry::File {
            rel: "docs/adr/.gitkeep",
            kind: FileKind::Static(include_str!("../templates/docs/adr/.gitkeep")),
        },
        PlanEntry::File {
            rel: ".agents/skills/feature-dev/SKILL.md",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/feature-dev/SKILL.md"
            )),
        },
        PlanEntry::File {
            rel: ".agents/skills/feature-dev/agents/openai.yaml",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/feature-dev/agents/openai.yaml"
            )),
        },
        PlanEntry::File {
            rel: ".agents/skills/git-commit/SKILL.md",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/git-commit/SKILL.md"
            )),
        },
        PlanEntry::File {
            rel: ".agents/skills/git-commit/agents/openai.yaml",
            kind: FileKind::Static(include_str!(
                "../templates/.agents/skills/git-commit/agents/openai.yaml"
            )),
        },
        PlanEntry::Symlink {
            rel: ".claude/skills",
            target: "../.agents/skills",
        },
        PlanEntry::File {
            rel: ".claude/settings.json",
            kind: FileKind::Static(include_str!("../templates/.claude/settings.json")),
        },
        PlanEntry::File {
            rel: ".codex/hooks.json",
            kind: FileKind::Static(include_str!("../templates/.codex/hooks.json")),
        },
        PlanEntry::File {
            rel: ".codex/hooks/spec_doc_review.py",
            kind: FileKind::Static(include_str!(
                "../templates/.codex/hooks/spec_doc_review.py"
            )),
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
