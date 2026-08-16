package plan

import "testing"

func TestPlanCoversRequiredPaths(t *testing.T) {
	entries, err := Harness()
	if err != nil {
		t.Fatal(err)
	}
	got := map[string]bool{}
	for _, e := range entries {
		got[e.Rel] = true
	}
	for _, want := range []string{
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
	} {
		if !got[want] {
			t.Fatalf("missing %s", want)
		}
	}
	if entries[1].Target != "AGENTS.md" {
		t.Fatalf("claude target %q", entries[1].Target)
	}
	var skills string
	for _, e := range entries {
		if e.Rel == ".claude/skills" {
			skills = e.Target
		}
	}
	if skills != "../.agents/skills" {
		t.Fatalf("skills target %q", skills)
	}
}
