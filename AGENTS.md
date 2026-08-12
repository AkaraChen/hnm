# Project workflows

## Feature development

For every new feature, use `$feature-dev` before implementation. Inspect the code and all project documentation, complete the one-question-at-a-time product and technical 质问, and record the agreed product requirements, architecture decisions, and general specification under `docs/` before changing feature code.

Documentation layout:

| Path | Role |
|------|------|
| `docs/prd/` | Product requirements: problem, users, goals/non-goals, flows, acceptance criteria |
| `docs/adr/` | Architecture decisions: context, choice, alternatives, trade-offs, failure bounds |
| `docs/spec.md` | Single source of truth for terminology, observable contracts, and system-wide invariants |

Read the relevant docs before changing feature code. Existing decisions live in `docs/adr/`, requirements in `docs/prd/`. Prefer updating those files over inventing parallel notes.

## Documentation rules

- Every completed feature 质问 leaves a PRD, at least one ADR when a material technical choice exists, and an update to `docs/spec.md` for any stable observable rule.
- `docs/spec.md` stays implementation-independent: shared terminology, externally observable contracts, and invariants. Put feature rationale in the PRD and decision rationale in ADRs.
- Use concise kebab-case filenames derived from the feature or decision.
- Prefer testable product statements in PRDs. Do not hide unresolved product questions inside implementation TODOs.
- If code and docs disagree, surface the conflict during 质问 or before commit; do not silently pick one side.

## Commit and ship

Before any commit, review staged, unstaged, and untracked changes against `docs/spec.md` and every relevant document under `docs/adr/` and `docs/prd/`. If the implementation changes a documented fact, update and stage the documentation in the same change. Use `$git-commit` to stage logical groups and create conventional commits from the diff.

# Commands

Package manager and build tooling are Cargo (Rust edition 2024).

- `cargo build` — compile the package
- `cargo test` — run unit and integration tests
- `cargo run -- init --help` — CLI help for harness install
- `cargo run -- init <path> --stack rust` — install harness into a path
- `cargo clippy` — lints
- `cargo fmt` — rustfmt

# Code style

- Prefer small modules with clear ownership boundaries over large catch-all files.
- Domain types and invariants belong in library modules (`init`, `plan`, `render`, `stack`), not only in `main`.
- Errors use `thiserror` in reusable code; `anyhow` only at the process edge.
- Harness layout lives in `plan::harness_plan` and embedded `templates/`; keep them in sync with `docs/spec.md`.
- Tests protect init contracts (full layout, dry-run, skip, force), not template trivia.

# Note

`CLAUDE.md` is a symlink to this file — edit `AGENTS.md`.
