// Package hooks owns hnm's agent-specific documentation review installation.
package hooks

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

//go:embed spec_doc_review.py
var script []byte

const command = `python3 "$(git rev-parse --show-toplevel)/.codex/hooks/spec_doc_review.py"`

type File struct {
	Path string
	Data []byte
	Mode os.FileMode
}

// Prepare validates existing files before init changes any harness content.
func Prepare(force bool) ([]File, error) {
	var files []File
	for _, path := range []string{".codex/hooks/spec_doc_review.py", ".claude/settings.json", ".codex/hooks.json"} {
		mode := os.FileMode(0o644)
		info, err := os.Lstat(path)
		exists := err == nil
		if err != nil && !os.IsNotExist(err) {
			return nil, err
		}
		if exists {
			if !info.Mode().IsRegular() {
				return nil, fmt.Errorf("%s must be a regular file", path)
			}
			mode = info.Mode().Perm()
		}
		if path == ".codex/hooks/spec_doc_review.py" {
			if !exists || force {
				files = append(files, File{path, script, mode})
			}
			continue
		}
		raw := []byte("{}")
		if exists {
			raw, err = os.ReadFile(path)
			if err != nil {
				return nil, err
			}
		}
		data, err := merge(raw)
		if err != nil {
			return nil, fmt.Errorf("%s: %w", path, err)
		}
		if !bytes.Equal(raw, data) {
			files = append(files, File{path, data, mode})
		}
	}
	return files, nil
}

func merge(raw []byte) ([]byte, error) {
	var settings map[string]json.RawMessage
	if err := json.Unmarshal(raw, &settings); err != nil {
		return nil, err
	}
	if settings == nil {
		return nil, fmt.Errorf("settings must be a JSON object")
	}
	events := map[string]json.RawMessage{}
	if value, ok := settings["hooks"]; ok {
		if err := json.Unmarshal(value, &events); err != nil {
			return nil, err
		}
		if events == nil {
			return nil, fmt.Errorf("hooks must be an object")
		}
	}
	var groups []json.RawMessage
	if value, ok := events["PreToolUse"]; ok {
		if err := json.Unmarshal(value, &groups); err != nil {
			return nil, err
		}
		if groups == nil {
			return nil, fmt.Errorf("PreToolUse must be an array")
		}
	}
	for _, rawGroup := range groups {
		var group struct {
			Matcher string `json:"matcher"`
			Hooks   []struct {
				Type    string `json:"type"`
				Command string `json:"command"`
			} `json:"hooks"`
		}
		if err := json.Unmarshal(rawGroup, &group); err != nil {
			return nil, err
		}
		if group.Matcher != "Bash" && group.Matcher != "^Bash$" {
			continue
		}
		for _, hook := range group.Hooks {
			if hook.Type == "command" && (hook.Command == command || hook.Command == "/usr/bin/"+command) {
				return raw, nil
			}
		}
	}
	group, err := json.Marshal(map[string]any{"matcher": "^Bash$", "hooks": []any{map[string]any{"type": "command", "command": command, "timeout": 10}}})
	if err != nil {
		return nil, err
	}
	groups = append(groups, group)
	events["PreToolUse"], err = json.Marshal(groups)
	if err != nil {
		return nil, err
	}
	settings["hooks"], err = json.Marshal(events)
	if err != nil {
		return nil, err
	}
	data, err := json.MarshalIndent(settings, "", "  ")
	return append(data, '\n'), err
}

// Write atomically replaces each file, preserving existing permission bits.
func (f File) Write() error {
	if err := os.MkdirAll(filepath.Dir(f.Path), 0o755); err != nil {
		return err
	}
	tmp, err := os.CreateTemp(filepath.Dir(f.Path), ".hnm-*")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())
	if err := tmp.Chmod(f.Mode); err != nil {
		tmp.Close()
		return err
	}
	if _, err := tmp.Write(f.Data); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	return os.Rename(tmp.Name(), f.Path)
}
