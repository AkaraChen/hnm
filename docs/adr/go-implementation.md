# ADR: Go is the only hnm implementation

## Status

Accepted. Supersedes `clap-minijinja-embedded-templates.md` for the current binary.

## Context

The Rust crate was kept only long enough to compare path set, symlink targets, headings, and stack commands. That comparison passed. Keeping both binaries makes install ambiguous.

## Decision

- Ship only the Go module `github.com/AkaraChen/hnm`.
- Install with `go install github.com/AkaraChen/hnm/cmd/hnm@latest`.
- Keep `templates/`, `schema/harness.json`, skills, and hooks.
- `--stack rust` remains a *target project* command preset, not an implementation language.

## Consequences

- `Cargo.toml`, `Cargo.lock`, and `src/` are removed.
- Product docs describe Go install and `hnm init` only.
