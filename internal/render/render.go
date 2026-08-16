package render

import (
	"strings"

	hnm "github.com/AkaraChen/hnm"
	"github.com/AkaraChen/hnm/internal/stack"
)

type Context struct {
	ProjectName string
	Stack       stack.Stack
}

func RenderAgents(ctx Context) (string, error) {
	raw, err := hnm.TemplateFS.ReadFile("templates/AGENTS.md.j2")
	if err != nil {
		return "", err
	}
	return apply(string(raw), ctx), nil
}

func RenderSpec(ctx Context) (string, error) {
	raw, err := hnm.TemplateFS.ReadFile("templates/docs/spec.md.j2")
	if err != nil {
		return "", err
	}
	return apply(string(raw), ctx), nil
}

func apply(src string, ctx Context) string {
	out := src
	out = strings.ReplaceAll(out, "{{ project_name }}", ctx.ProjectName)
	out = strings.ReplaceAll(out, "{{ commands }}", ctx.Stack.CommandsBlock())
	out = strings.ReplaceAll(out, "{{ stack }}", ctx.Stack.String())
	return out
}
