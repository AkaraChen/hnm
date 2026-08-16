package initcmd

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/AkaraChen/hnm/internal/stack"
)

func TestResolveNamePrefersExplicit(t *testing.T) {
	if got := ResolveProjectName("  demo  ", "/tmp/other"); got != "demo" {
		t.Fatalf("got %q", got)
	}
}

func TestResolveNameFromDir(t *testing.T) {
	if got := ResolveProjectName("", "/tmp/my-app"); got != "my-app" {
		t.Fatalf("got %q", got)
	}
}

func TestInitWritesFullHarness(t *testing.T) {
	dir := t.TempDir()
	report, err := Run(Options{Target: dir, ProjectName: "sample", Stack: stack.Rust})
	if err != nil {
		t.Fatal(err)
	}
	for _, a := range report.Actions {
		if a.Kind != ActionCreate && a.Kind != ActionLink {
			t.Fatalf("unexpected %s %s", a.Kind, a.Rel)
		}
	}
	agents, err := os.ReadFile(filepath.Join(dir, "AGENTS.md"))
	if err != nil {
		t.Fatal(err)
	}
	text := string(agents)
	if !strings.Contains(text, "`sample`") || !strings.Contains(text, "cargo test") {
		t.Fatalf("agents missing name or cargo: %s", text[:200])
	}
	skill := filepath.Join(dir, ".agents/skills/feature-dev/SKILL.md")
	body, err := os.ReadFile(skill)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(body), "\u8d28\u95ee") {
		t.Fatal("feature-dev skill missing question mark word")
	}
	gitCommit := filepath.Join(dir, ".agents/skills/git-commit/SKILL.md")
	gc, err := os.ReadFile(gitCommit)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(gc), "Conventional Commits") {
		t.Fatal("git-commit skill missing Conventional Commits")
	}
	if _, err := os.Stat(filepath.Join(dir, ".agents/skills/git-commit/agents/openai.yaml")); err != nil {
		t.Fatal(err)
	}
	claude, err := os.Readlink(filepath.Join(dir, "CLAUDE.md"))
	if err != nil {
		t.Fatal(err)
	}
	if claude != "AGENTS.md" {
		t.Fatalf("CLAUDE.md -> %q", claude)
	}
	skills, err := os.Readlink(filepath.Join(dir, ".claude/skills"))
	if err != nil {
		t.Fatal(err)
	}
	if skills != "../.agents/skills" {
		t.Fatalf("skills -> %q", skills)
	}
	for _, rel := range []string{".codex/hooks/spec_doc_review.py", "docs/prd/.gitkeep", "docs/adr/.gitkeep"} {
		if _, err := os.Stat(filepath.Join(dir, rel)); err != nil {
			t.Fatalf("%s: %v", rel, err)
		}
	}
}

func TestDryRunWritesNothing(t *testing.T) {
	dir := t.TempDir()
	if _, err := Run(Options{Target: dir, ProjectName: "x", Stack: stack.Generic, DryRun: true}); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(filepath.Join(dir, "AGENTS.md")); !os.IsNotExist(err) {
		t.Fatal("dry-run wrote AGENTS.md")
	}
}

func TestSkipWithoutForce(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "AGENTS.md")
	if err := os.WriteFile(path, []byte("keep-me\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	report, err := Run(Options{Target: dir, ProjectName: "x", Stack: stack.Generic})
	if err != nil {
		t.Fatal(err)
	}
	var agents ActionReport
	for _, a := range report.Actions {
		if a.Rel == "AGENTS.md" {
			agents = a
		}
	}
	if agents.Kind != ActionSkipExists {
		t.Fatalf("kind %s", agents.Kind)
	}
	got, _ := os.ReadFile(path)
	if string(got) != "keep-me\n" {
		t.Fatalf("rewrote file: %q", got)
	}
}

func TestForceOverwrites(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "AGENTS.md"), []byte("old\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, err := Run(Options{Target: dir, ProjectName: "forced", Stack: stack.Bun, Force: true}); err != nil {
		t.Fatal(err)
	}
	agents, err := os.ReadFile(filepath.Join(dir, "AGENTS.md"))
	if err != nil {
		t.Fatal(err)
	}
	text := string(agents)
	if !strings.Contains(text, "`forced`") || !strings.Contains(text, "bun test") || strings.HasPrefix(text, "old") {
		t.Fatalf("force overwrite failed: %s", text[:200])
	}
}
