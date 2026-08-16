package stack

import "testing"

func TestRustBlockMentionsCargo(t *testing.T) {
	if !contains(Rust.CommandsBlock(), "cargo test") {
		t.Fatal("rust block should mention cargo test")
	}
}

func TestParseStackNames(t *testing.T) {
	got, err := Parse("bun")
	if err != nil || got != Bun {
		t.Fatalf("parse bun: %v %q", err, got)
	}
	if _, err := Parse("nope"); err == nil {
		t.Fatal("expected unknown stack")
	}
}

func contains(s, sub string) bool {
	return len(s) >= len(sub) && (s == sub || indexOf(s, sub) >= 0)
}

func indexOf(s, sub string) int {
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return i
		}
	}
	return -1
}
