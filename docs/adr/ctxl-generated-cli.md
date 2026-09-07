# Generate the hnm CLI from a ctxl schema

Status: Accepted (supersedes clap-minijinja-embedded-templates.md and the frontmatter-avoidance decision in prd/go-init-drift.md)

## Context

hnm maintained a bespoke Go implementation (cobra wiring, embedded templates, `--stack` presets, plan/render/init packages, ~700 lines plus tests) whose data model already existed as a ctxl schema (`schema/harness.json`). ctxl now generates complete schema-specialized CLIs, so the duplication no longer pays for itself.

## Decision

- Declare the whole harness in a root `context.schema.json` and generate `internal/generated` with ctxl in existing-module mode; the output is generated-owned and replaced in full.
- Pin generation to one ctxl commit in both `generation.ctxl_version` and the `justfile`'s single `generate` recipe; never generate from `latest`.
- Reuse `.agents/skills/` both as this repository's development harness and as the bundled skill payload (`hnm skills list|get|path`).
- Drop template rendering (`--name`, `--stack` presets, `--dry-run`) and the `.claude/skills` directory symlink. KIT-928 restores hook/settings installation in a hand-written hnm entrypoint.

## Alternatives considered

### Keep the bespoke implementation

Rejected: it re-implements what ctxl generates, and its content-rendering value (stack presets, seeded AGENTS.md text) does not justify maintaining a parallel CLI.

### Generate a standalone module

Rejected: `go install github.com/AkaraChen/hnm/cmd/hnm@latest` must keep working, which existing-module mode preserves without touching `go.mod` ownership.

## Consequences

- `hnm init` creates the five declared harness paths, seeding AGENTS.md and spec from schema bodies.
- Fieldless singular Markdown is plain text; entities with declared metadata retain YAML frontmatter.
- hnm installs the documentation-review hook for Claude and Codex; agents load bundled skills via `hnm skills`.
- CLI fixes and new entity behavior arrive by bumping the pinned ctxl commit and regenerating.

## hnm extension boundary (KIT-928)

Generate an importable `internal/generated` package exposing `New()`, then compose it from the user-owned `cmd/hnm` entrypoint. Wrap only the generated init command; use its existing store initialization and command tree. Keep the legacy Python script embedded in hnm and merge JSON settings using Go's standard library. Do not put agent-specific hooks into ctxl or patch generated source. This keeps regeneration reproducible and hook policy local. Retain skip/force script ownership while merging settings to avoid erasing user configuration. Tests exercise the composed command and invoke the script with real hook payloads.
