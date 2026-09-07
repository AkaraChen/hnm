package main

import (
	"bytes"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

func runCLI(args ...string) (string, error) {
	cmd := newCommand()
	var out bytes.Buffer
	cmd.SetOut(&out)
	cmd.SetErr(&out)
	cmd.SetArgs(args)
	err := cmd.Execute()
	return out.String(), err
}

func readFile(t *testing.T, path string) []byte {
	t.Helper()
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	return raw
}

func TestInitAndEntityCommands(t *testing.T) {
	t.Chdir(t.TempDir())
	for _, path := range []string{".claude/settings.json", ".codex/hooks.json"} {
		if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(path, []byte(`{"large":9007199254740993,"hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo stop"}]}],"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"echo custom"}]}]}}`), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	if out, err := runCLI("init"); err != nil {
		t.Fatalf("%s: %v", out, err)
	}
	if body := string(readFile(t, "AGENTS.md")); !strings.HasPrefix(body, "# Project workflows") || !strings.Contains(body, "hnm skills get feature-dev") {
		t.Fatal(body)
	}
	if target, err := os.Readlink("CLAUDE.md"); err != nil || target != "AGENTS.md" {
		t.Fatalf("link = %q, %v", target, err)
	}
	if out, err := runCLI("agents", "show"); err != nil || !strings.Contains(out, "Project workflows") {
		t.Fatalf("show = %s, %v", out, err)
	}
	if out, err := runCLI("agents", "write", "--body", "# Custom\n"); err != nil {
		t.Fatalf("%s: %v", out, err)
	}
	if got := string(readFile(t, "AGENTS.md")); got != "# Custom\n" {
		t.Fatal(got)
	}
	before := map[string][]byte{}
	for _, path := range []string{"AGENTS.md", ".claude/settings.json", ".codex/hooks.json", ".codex/hooks/spec_doc_review.py"} {
		before[path] = readFile(t, path)
	}
	for _, force := range []bool{false, true} {
		args := []string{"init"}
		if force {
			args = append(args, "--force")
		}
		if out, err := runCLI(args...); err != nil {
			t.Fatalf("%s: %v", out, err)
		}
		for path, raw := range before {
			if path == "AGENTS.md" && force {
				continue
			}
			if !bytes.Equal(raw, readFile(t, path)) {
				t.Fatalf("repeat init changed %s", path)
			}
		}
	}
	if !strings.HasPrefix(string(readFile(t, "AGENTS.md")), "# Project workflows") {
		t.Fatal("force did not reset seed")
	}
	for _, path := range []string{".claude/settings.json", ".codex/hooks.json"} {
		raw := readFile(t, path)
		if !bytes.Contains(raw, []byte("9007199254740993")) || !bytes.Contains(raw, []byte("echo stop")) || !bytes.Contains(raw, []byte("echo custom")) {
			t.Fatalf("lost user settings: %s", raw)
		}
		if n := bytes.Count(raw, []byte("spec_doc_review.py")); n != 1 {
			t.Fatalf("hook count = %d", n)
		}
		if info, err := os.Stat(path); err != nil || info.Mode().Perm() != 0o600 {
			t.Fatalf("permissions: %v, %v", info, err)
		}
	}
	if err := os.WriteFile(".codex/hooks/spec_doc_review.py", []byte("custom script"), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, err := runCLI("init"); err != nil {
		t.Fatal(err)
	}
	if string(readFile(t, ".codex/hooks/spec_doc_review.py")) != "custom script" {
		t.Fatal("overwrote existing script")
	}
	if _, err := runCLI("init", "--force"); err != nil {
		t.Fatal(err)
	}
	if !bytes.Contains(readFile(t, ".codex/hooks/spec_doc_review.py"), []byte("def main")) {
		t.Fatal("force did not restore script")
	}
	if _, err := runCLI("init", "ignored-path"); err == nil {
		t.Fatal("accepted ignored init path")
	}
}

func TestInvalidSettingsDoNotChangeHarness(t *testing.T) {
	for _, raw := range []string{"{", "null", "[]", `{"hooks":null}`, `{"hooks":[]}`, `{"hooks":{"PreToolUse":{}}}`} {
		t.Run(raw, func(t *testing.T) {
			t.Chdir(t.TempDir())
			if err := os.MkdirAll(".claude", 0o755); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(".claude/settings.json", []byte(raw), 0o644); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile("AGENTS.md", []byte("keep"), 0o644); err != nil {
				t.Fatal(err)
			}
			if _, err := runCLI("init", "--force"); err == nil {
				t.Fatal("accepted invalid settings")
			}
			if string(readFile(t, "AGENTS.md")) != "keep" || string(readFile(t, ".claude/settings.json")) != raw {
				t.Fatal("changed files on invalid input")
			}
		})
	}
}

func TestInstalledReviewHook(t *testing.T) {
	t.Setenv("SPEC_DOC_REVIEW_STATE_DIR", t.TempDir())
	t.Chdir(t.TempDir())
	cwd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	if out, err := exec.Command("git", "init", "-q").CombinedOutput(); err != nil {
		t.Fatalf("%s: %v", out, err)
	}
	if _, err := runCLI("init"); err != nil {
		t.Fatal(err)
	}
	// Exercise the actual installed command, as both runtimes match shell calls as Bash.
	var settings struct {
		Hooks map[string][]struct {
			Hooks []struct {
				Command string `json:"command"`
			} `json:"hooks"`
		} `json:"hooks"`
	}
	if err := json.Unmarshal(readFile(t, ".codex/hooks.json"), &settings); err != nil {
		t.Fatal(err)
	}
	command := settings.Hooks["PreToolUse"][0].Hooks[0].Command
	invoke := func(input, session string) string {
		t.Helper()
		payload, err := json.Marshal(map[string]any{"cwd": cwd, "session_id": session, "tool_name": "Bash", "tool_input": map[string]string{"command": input}})
		if err != nil {
			t.Fatal(err)
		}
		cmd := exec.Command("sh", "-c", command)
		cmd.Stdin = bytes.NewReader(payload)
		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("hook: %s: %v", out, err)
		}
		return string(out)
	}
	if got := invoke("git status", "one"); got != "" {
		t.Fatal(got)
	}
	if got := invoke("git commit -m example", "one"); !strings.Contains(got, `"permissionDecision": "deny"`) || !strings.Contains(got, "docs/spec.md") {
		t.Fatal(got)
	}
	if got := invoke("git commit -m example", "one"); got != "" {
		t.Fatal(got)
	}
	if got := invoke("git commit -m example", "two"); !strings.Contains(got, `"permissionDecision": "deny"`) {
		t.Fatal(got)
	}
	if out, err := exec.Command("git", "-c", "user.name=Test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "test").CombinedOutput(); err != nil {
		t.Fatalf("%s: %v", out, err)
	}
	if got := invoke("git commit -m next", "one"); !strings.Contains(got, `"permissionDecision": "deny"`) {
		t.Fatal(got)
	}
}
