# Specification

## Product scope

`hnm` is a CLI that installs a complete agent documentation harness into a target project directory. The harness is a standard three-layer layout: docs (`docs/prd/`, `docs/adr/`, `docs/spec.md`), root `AGENTS.md` workflow rules, installed agent skills (`feature-dev` 质问 and `git-commit`), Claude/Codex skill wiring, and a pre-commit documentation review gate.

The tool does not scaffold application business code, package managers, CI, or git repositories.

## Terminology

- **Harness**: the fixed set of agent workflow files and directories that `hnm init` writes.
- **Feature 质问**: the mandatory product-then-technical clarification loop defined by the installed `$feature-dev` skill.
- **PRD**: a product requirements document under `docs/prd/`.
- **ADR**: an architecture decision record under `docs/adr/`.
- **Spec**: `docs/spec.md` — shared terminology, observable contracts, and system-wide invariants for the *target* project after init (and for this repository as the tool itself).
- **Stack**: a named command-block preset used only to fill the Commands section of generated `AGENTS.md` (`rust`, `node`, `bun`, `go`, `python`, `generic`).
- **Init plan**: the ordered list of create/skip/link actions computed before writing.

## Observable contracts

### CLI

- The binary name is `hnm`.
- Primary command: `hnm init [PATH]`.
  - `PATH` defaults to `.`.
  - `--name <NAME>` sets the project name embedded in templates; when omitted, use the target directory’s final path component (or `project` if it is empty/`.`-only after canonicalize fallback to the user-facing path name).
  - `--stack <STACK>` selects the Commands section preset; default `generic`.
  - `--force` overwrites existing regular files and replaces incorrect symlinks.
  - `--dry-run` prints the plan without writing.
- Unknown subcommands or invalid flags exit non-zero with cobra error output.
- On success, stdout includes a human-readable summary of created, skipped, and linked paths.

### Generated harness layout

After a successful full init (no skips), the target directory contains at least:

| Path | Kind |
|------|------|
| `AGENTS.md` | file (rendered) |
| `CLAUDE.md` | symlink → `AGENTS.md` |
| `docs/spec.md` | file (rendered) |
| `docs/prd/.gitkeep` | file |
| `docs/adr/.gitkeep` | file |
| `.agents/skills/feature-dev/SKILL.md` | file |
| `.agents/skills/feature-dev/agents/openai.yaml` | file |
| `.agents/skills/git-commit/SKILL.md` | file |
| `.agents/skills/git-commit/agents/openai.yaml` | file |
| `.claude/skills` | symlink → `../.agents/skills` |
| `.claude/settings.json` | file |
| `.codex/hooks.json` | file |
| `.codex/hooks/spec_doc_review.py` | file |

### Generation rules

- Parameterized files are rendered by replacing {{ project_name }}, {{ commands }}, and {{ stack }}; static assets are written byte-identical to the embedded resources.
- Without `--force`, existing regular files are not overwritten; those paths are reported as skipped.
- With `--force`, existing regular files at plan paths are replaced with generated content.
- Symlink targets are relative as listed above.
- Missing parent directories are created as needed.
- If `PATH` does not exist, init creates it as a directory when possible.
- Dry-run performs no filesystem mutations.

### Installed workflow rules (target `AGENTS.md`)

Generated `AGENTS.md` requires:

- use `$feature-dev` before implementing new features;
- record PRD, ADR, and spec updates under `docs/` before feature code;
- review docs against the working tree before commit;
- use `$git-commit` for conventional commits from the diff.

The installed `feature-dev` skill, `git-commit` skill, and `spec_doc_review` hook implement those rules for agent runtimes that load project skills/hooks.

## System-wide constraints

- Language: Go.
- CLI: cobra.
- Templates and static assets are embedded at compile time from `templates/`.
- Contract symlink path/target for `CLAUDE.md` is declared in `schema/harness.json` and read through ctxl schema.
- Large packages are not allowed; split by responsibility (`cmd/hnm`, `internal/stack`, `internal/plan`, `internal/render`, `internal/init`).
- This repository dogfoods the same harness layout for developing `hnm` itself.

## Current implementation status

- Product scope and init contract are specified.
- Implementation delivers `hnm init` with cobra, embedded templates, skip/force/dry-run, and automated tests.
- The former Rust crate has been removed; Go is the only implementation.
