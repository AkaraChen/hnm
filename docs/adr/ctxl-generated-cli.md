# Generate the hnm CLI from a ctxl schema

Status: Accepted (supersedes clap-minijinja-embedded-templates.md and the frontmatter-avoidance decision in prd/go-init-drift.md)

## Context

hnm maintained a bespoke Go implementation (cobra wiring, embedded templates, `--stack` presets, plan/render/init packages, ~700 lines plus tests) whose data model already existed as a ctxl schema (`schema/harness.json`). ctxl now generates complete schema-specialized CLIs, so the duplication no longer pays for itself.

## Decision

- Declare the whole harness in a root `context.schema.json` and generate `cmd/hnm` with ctxl in existing-module mode; the output is generated-owned and replaced in full.
- Pin generation to one ctxl commit in both `generation.ctxl_version` and the `justfile`'s single `generate` recipe; never generate from `latest`.
- Reuse `.agents/skills/` both as this repository's development harness and as the bundled skill payload (`hnm skills list|get|path`).
- Drop template rendering (`--name`, `--stack` presets, `--dry-run`), hook/settings installation, and the `.claude/skills` directory symlink from the product surface.

## Alternatives considered

### Keep the bespoke implementation

Rejected: it re-implements what ctxl generates, and its content-rendering value (stack presets, seeded AGENTS.md text) does not justify maintaining a parallel CLI.

### Generate a standalone module

Rejected: `go install github.com/AkaraChen/hnm/cmd/hnm@latest` must keep working, which existing-module mode preserves without touching `go.mod` ownership.

## Consequences

- `hnm init` now creates the five declared harness paths with empty frontmatter instead of rendered AGENTS.md/spec content; authoring moves to the entity commands or the editor.
- Entity-written markdown carries YAML frontmatter, reversing go-init-drift's frontmatter avoidance.
- Hook and settings files are no longer installed into target projects; agent runtimes load the bundled skills via `hnm skills`.
- CLI fixes and new entity behavior arrive by bumping the pinned ctxl commit and regenerating.
