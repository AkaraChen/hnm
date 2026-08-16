package plan

import (
	"fmt"

	"github.com/AkaraChen/ctxl/schema"
	hnm "github.com/AkaraChen/hnm"
)

type Kind int

const (
	KindFile Kind = iota
	KindSymlink
)

type FileSource int

const (
	SourceAgents FileSource = iota
	SourceSpec
	SourceStatic
)

type Entry struct {
	Rel    string
	Target string
	Kind   Kind
	Source FileSource
	Body   string
}

func Harness() ([]Entry, error) {
	s, err := schema.Parse(hnm.HarnessSchema)
	if err != nil {
		return nil, err
	}
	claude, err := s.Entity("claude")
	if err != nil {
		return nil, err
	}
	static := []struct {
		rel string
		src string
	}{
		{"docs/prd/.gitkeep", "templates/docs/prd/.gitkeep"},
		{"docs/adr/.gitkeep", "templates/docs/adr/.gitkeep"},
		{".agents/skills/feature-dev/SKILL.md", "templates/.agents/skills/feature-dev/SKILL.md"},
		{".agents/skills/feature-dev/agents/openai.yaml", "templates/.agents/skills/feature-dev/agents/openai.yaml"},
		{".agents/skills/git-commit/SKILL.md", "templates/.agents/skills/git-commit/SKILL.md"},
		{".agents/skills/git-commit/agents/openai.yaml", "templates/.agents/skills/git-commit/agents/openai.yaml"},
		{".claude/settings.json", "templates/.claude/settings.json"},
		{".codex/hooks.json", "templates/.codex/hooks.json"},
		{".codex/hooks/spec_doc_review.py", "templates/.codex/hooks/spec_doc_review.py"},
	}
	out := []Entry{
		{Rel: "AGENTS.md", Kind: KindFile, Source: SourceAgents},
		{Rel: claude.Path, Target: claude.Target, Kind: KindSymlink},
		{Rel: "docs/spec.md", Kind: KindFile, Source: SourceSpec},
	}
	for _, item := range static[:2] {
		body, err := readTemplate(item.src)
		if err != nil {
			return nil, err
		}
		out = append(out, Entry{Rel: item.rel, Kind: KindFile, Source: SourceStatic, Body: body})
	}
	for _, item := range static[2:6] {
		body, err := readTemplate(item.src)
		if err != nil {
			return nil, err
		}
		out = append(out, Entry{Rel: item.rel, Kind: KindFile, Source: SourceStatic, Body: body})
	}
	out = append(out, Entry{Rel: ".claude/skills", Target: "../.agents/skills", Kind: KindSymlink})
	for _, item := range static[6:] {
		body, err := readTemplate(item.src)
		if err != nil {
			return nil, err
		}
		out = append(out, Entry{Rel: item.rel, Kind: KindFile, Source: SourceStatic, Body: body})
	}
	return out, nil
}

func readTemplate(name string) (string, error) {
	raw, err := hnm.TemplateFS.ReadFile(name)
	if err != nil {
		return "", fmt.Errorf("template %s: %w", name, err)
	}
	return string(raw), nil
}
