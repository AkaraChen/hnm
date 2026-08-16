# hnm

Install a complete **agent documentation harness** into any project:

- `docs/prd/`, `docs/adr/`, `docs/spec.md`
- `AGENTS.md` + `CLAUDE.md` symlink
- `$feature-dev` skill (one-question loop before implementation)
- `$git-commit` skill (conventional commits from the diff)
- Claude/Codex skill wiring and pre-commit documentation review gate

## Install

```bash
go install github.com/AkaraChen/hnm/cmd/hnm@latest
```

## Usage

```bash
hnm init
hnm init ./my-app --name my-app --stack go
hnm init --dry-run
hnm init --force
```

### Stack presets

See --stack generic, rust, node, bun, go, python.

## What gets written

AGENTS.md, CLAUDE.md (symlink), docs/spec.md, docs/prd/, docs/adr/, .agents/skills, .claude/skills (symlink), hooks.

## Develop

    go test ./...
    go run ./cmd/hnm init --help
