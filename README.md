# hnm

Install a complete **agent documentation harness** into any project:

- `docs/prd/`, `docs/adr/`, `docs/spec.md`
- `AGENTS.md` + `CLAUDE.md` symlink
- `$feature-dev` skill (one-question 质问 before implementation)
- `$git-commit` skill (conventional commits from the diff)
- Claude/Codex skill wiring and pre-commit documentation review gate

## Install

```bash
go install github.com/AkaraChen/hnm/cmd/hnm@latest
```

The Rust crate remains in this repository for comparison until the Go binary is the default:

```bash
cargo install --path .
```

```bash
go install github.com/AkaraChen/hnm/cmd/hnm@latest
```

Rust remains the comparison binary until the Go port is the default.

## Usage

```bash
# Install into the current directory
hnm init

# Target path, name, and stack preset
hnm init ./my-app --name my-app --stack rust

# Preview without writing
hnm init --dry-run

# Overwrite existing harness files
hnm init --force
```

### Stack presets

| `--stack` | Commands section |
|-----------|------------------|
| `generic` (default) | Placeholder notes |
| `rust` | cargo build/test/clippy/fmt |
| `node` | npm scripts |
| `bun` | bun scripts |
| `go` | go test/build/vet |
| `python` | pytest/ruff/mypy when configured |

## What gets written

| Path | Notes |
|------|--------|
| `AGENTS.md` | Workflow + docs rules + stack commands |
| `CLAUDE.md` | → `AGENTS.md` |
| `docs/spec.md` | Bootstrap specification |
| `docs/prd/`, `docs/adr/` | Empty dirs with `.gitkeep` |
| `.agents/skills/feature-dev/` | Full 质问 skill |
| `.agents/skills/git-commit/` | Conventional commit skill |
| `.claude/skills` | → `../.agents/skills` |
| `.claude/settings.json` | Commit doc-review hook |
| `.codex/hooks.json` + `spec_doc_review.py` | Codex commit gate |

## Develop

```bash
cargo test
cargo run -- init --help
```
