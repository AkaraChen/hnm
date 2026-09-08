package main

import (
	"bytes"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"github.com/AkaraChen/hnm/internal/generated"
	"github.com/AkaraChen/hnm/internal/hooks"
)

func installSkills(force bool, out io.Writer) error {
	for _, name := range []string{"feature-dev", "git-commit"} {
		// Reuse the generated package's complete skill materialization.
		command := generated.New()
		var result bytes.Buffer
		command.SetOut(&result)
		command.SetErr(out)
		command.SetArgs([]string{"skills", "path", name})
		if err := command.Execute(); err != nil {
			return err
		}
		source := strings.TrimSpace(result.String())
		target := filepath.Join(".agents", "skills", name)
		if err := filepath.WalkDir(source, func(path string, entry fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			rel, err := filepath.Rel(source, path)
			if err != nil {
				return err
			}
			dest := filepath.Join(target, rel)
			info, err := entry.Info()
			if err != nil {
				return err
			}
			if entry.IsDir() {
				if existing, err := os.Lstat(dest); err == nil && existing.Mode()&os.ModeSymlink != 0 {
					if !force {
						return fs.SkipDir
					}
					return fmt.Errorf("refusing to write through skill directory symlink %s", dest)
				}
				return os.MkdirAll(dest, info.Mode().Perm())
			}
			if !info.Mode().IsRegular() {
				return fmt.Errorf("unsupported bundled file %s", path)
			}
			if existing, err := os.Lstat(dest); err == nil {
				if !force {
					return nil
				}
				if !existing.Mode().IsRegular() {
					return fmt.Errorf("%s must be a regular file", dest)
				}
			} else if !os.IsNotExist(err) {
				return err
			}
			data, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			return (hooks.File{Path: dest, Data: data, Mode: info.Mode().Perm()}).Write()
		}); err != nil {
			return err
		}
		alias := filepath.Join(".claude", "skills", name)
		link := filepath.Join("..", "..", target)
		if existing, err := os.Lstat(alias); err == nil {
			// A legacy whole-directory alias already exposes the installed skill.
			if same, err := os.Stat(target); err == nil && os.SameFile(existing, same) {
				continue
			}
			if current, err := os.Readlink(alias); err == nil && current == link {
				continue
			}
			if !force {
				fmt.Fprintf(out, "skipped existing %s\n", alias)
				continue
			}
			if err := os.Remove(alias); err != nil {
				return err
			}
		} else if !os.IsNotExist(err) {
			return err
		}
		if err := os.MkdirAll(filepath.Dir(alias), 0o755); err != nil {
			return err
		}
		if err := os.Symlink(link, alias); err != nil {
			return err
		}
		fmt.Fprintf(out, "installed skill %s\n", name)
	}
	return nil
}
