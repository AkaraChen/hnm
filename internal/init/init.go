package initcmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/AkaraChen/hnm/internal/plan"
	"github.com/AkaraChen/hnm/internal/render"
	"github.com/AkaraChen/hnm/internal/stack"
)

type Options struct {
	Target      string
	ProjectName string
	Stack       stack.Stack
	Force       bool
	DryRun      bool
}

type ActionKind string

const (
	ActionCreate     ActionKind = "create"
	ActionOverwrite  ActionKind = "overwrite"
	ActionSkipExists ActionKind = "skip"
	ActionLink       ActionKind = "link"
	ActionRelink     ActionKind = "relink"
	ActionSkipLinkOk ActionKind = "skip-link"
)

type ActionReport struct {
	Rel  string
	Kind ActionKind
}

type Report struct {
	Actions []ActionReport
}

func (r Report) SummaryLines() []string {
	out := make([]string, 0, len(r.Actions))
	for _, a := range r.Actions {
		out = append(out, fmt.Sprintf("%-10s %s", a.Kind, a.Rel))
	}
	return out
}

func ResolveProjectName(explicit string, target string) string {
	if name := strings.TrimSpace(explicit); name != "" {
		return name
	}
	base := strings.TrimSpace(filepath.Base(target))
	if base != "" && base != "." && base != ".." {
		return base
	}
	return "project"
}

func Run(opts Options) (Report, error) {
	if err := ensureTargetDir(opts.Target); err != nil {
		return Report{}, err
	}
	entries, err := plan.Harness()
	if err != nil {
		return Report{}, err
	}
	ctx := render.Context{ProjectName: opts.ProjectName, Stack: opts.Stack}
	var actions []ActionReport
	for _, entry := range entries {
		switch entry.Kind {
		case plan.KindFile:
			content, err := fileContent(entry, ctx)
			if err != nil {
				return Report{}, err
			}
			rep, err := writeFile(opts.Target, entry.Rel, content, opts.Force, opts.DryRun)
			if err != nil {
				return Report{}, err
			}
			actions = append(actions, rep)
		case plan.KindSymlink:
			rep, err := writeSymlink(opts.Target, entry.Rel, entry.Target, opts.Force, opts.DryRun)
			if err != nil {
				return Report{}, err
			}
			actions = append(actions, rep)
		}
	}
	return Report{Actions: actions}, nil
}

func fileContent(entry plan.Entry, ctx render.Context) (string, error) {
	switch entry.Source {
	case plan.SourceAgents:
		return render.RenderAgents(ctx)
	case plan.SourceSpec:
		return render.RenderSpec(ctx)
	default:
		return entry.Body, nil
	}
}

func ensureTargetDir(target string) error {
	info, err := os.Stat(target)
	if err == nil {
		if info.IsDir() {
			return nil
		}
		return fmt.Errorf("target path is not a directory: %s", target)
	}
	if !os.IsNotExist(err) {
		return err
	}
	if err := os.MkdirAll(target, 0o755); err != nil {
		return fmt.Errorf("failed to create directory %s: %w", target, err)
	}
	return nil
}

func pathExists(path string) bool {
	if _, err := os.Lstat(path); err == nil {
		return true
	}
	return false
}

func writeFile(root, rel, content string, force, dryRun bool) (ActionReport, error) {
	path := filepath.Join(root, rel)
	exists := pathExists(path)
	if exists && !force {
		return ActionReport{Rel: rel, Kind: ActionSkipExists}, nil
	}
	kind := ActionCreate
	if exists {
		kind = ActionOverwrite
	}
	if dryRun {
		return ActionReport{Rel: rel, Kind: kind}, nil
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return ActionReport{}, fmt.Errorf("failed to create directory %s: %w", filepath.Dir(path), err)
	}
	if exists {
		if err := os.Remove(path); err != nil {
			if err2 := os.RemoveAll(path); err2 != nil {
				return ActionReport{}, fmt.Errorf("failed to remove %s: %w", path, err)
			}
		}
	}
	f, err := os.Create(path)
	if err != nil {
		return ActionReport{}, fmt.Errorf("failed to write %s: %w", path, err)
	}
	defer f.Close()
	if _, err := f.WriteString(content); err != nil {
		return ActionReport{}, fmt.Errorf("failed to write %s: %w", path, err)
	}
	if content != "" && !strings.HasSuffix(content, "\n") {
		if _, err := f.WriteString("\n"); err != nil {
			return ActionReport{}, fmt.Errorf("failed to write %s: %w", path, err)
		}
	}
	return ActionReport{Rel: rel, Kind: kind}, nil
}

func writeSymlink(root, rel, target string, force, dryRun bool) (ActionReport, error) {
	linkPath := filepath.Join(root, rel)
	if info, err := os.Lstat(linkPath); err == nil {
		if info.Mode()&os.ModeSymlink != 0 {
			if cur, err := os.Readlink(linkPath); err == nil && cur == target {
				return ActionReport{Rel: rel, Kind: ActionSkipLinkOk}, nil
			}
		}
		if !force {
			return ActionReport{Rel: rel, Kind: ActionSkipExists}, nil
		}
	}
	exists := pathExists(linkPath)
	kind := ActionLink
	if exists {
		kind = ActionRelink
	}
	if dryRun {
		return ActionReport{Rel: rel, Kind: kind}, nil
	}
	if err := os.MkdirAll(filepath.Dir(linkPath), 0o755); err != nil {
		return ActionReport{}, fmt.Errorf("failed to create directory %s: %w", filepath.Dir(linkPath), err)
	}
	if exists {
		info, err := os.Lstat(linkPath)
		if err != nil {
			return ActionReport{}, err
		}
		if info.IsDir() && info.Mode()&os.ModeSymlink == 0 {
			if err := os.RemoveAll(linkPath); err != nil {
				return ActionReport{}, fmt.Errorf("failed to remove %s: %w", linkPath, err)
			}
		} else if err := os.Remove(linkPath); err != nil {
			return ActionReport{}, fmt.Errorf("failed to remove %s: %w", linkPath, err)
		}
	}
	if err := os.Symlink(target, linkPath); err != nil {
		return ActionReport{}, fmt.Errorf("failed to create symlink %s -> %s: %w", linkPath, target, err)
	}
	return ActionReport{Rel: rel, Kind: kind}, nil
}
