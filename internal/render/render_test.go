package render

import (
	"strings"
	"testing"

	"github.com/AkaraChen/hnm/internal/stack"
)

func TestAgentsIncludesNameAndRustCommands(t *testing.T) {
	out, err := RenderAgents(Context{ProjectName: "demo", Stack: stack.Rust})
	if err != nil {
		t.Fatal(err)
	}
	for _, want := range []string{"`demo`", "cargo test", "$feature-dev", "$git-commit"} {
		if !strings.Contains(out, want) {
			t.Fatalf("agents missing %q", want)
		}
	}
}

func TestSpecIncludesProjectName(t *testing.T) {
	out, err := RenderSpec(Context{ProjectName: "acme", Stack: stack.Generic})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(out, "`acme`") || !strings.Contains(out, "docs/prd/") {
		t.Fatalf("spec missing name or docs/prd/: %s", out)
	}
}
