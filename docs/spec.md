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
- **Init plan**: the ordered list of create/update/unchanged/skip/link actions computed before writing.
- **Managed block**: the region between `<!-- hnm:begin -->` and `<!-- hnm:end -->` that hnm owns inside a user's Markdown file.

## Observable contracts

### CLI

- The binary name is `hnm`.
- Primary command: `hnm init [PATH]`.
  - `PATH` defaults to `.`.
  - `--name <NAME>` sets the project name embedded in templates; when omitted, use the supplied target path’s final component, falling back to `project` when absent, empty, `.` or `..`. The default `.` path therefore embeds `project`; the CLI does not canonicalize it to infer the current directory name.
  - `--stack <STACK>` selects the Commands section preset; default `generic`.
  - `--dry-run` prints the plan without writing.
- Unknown subcommands or invalid flags exit non-zero with clap’s error output.
- On success, stdout includes a human-readable summary of created, updated, unchanged, skipped, and linked paths, followed by a note counting skipped paths when any exist. Skips do not change the exit code.

### Generated harness layout

After a successful init into an empty directory, the target directory contains at least:

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
| `.codex/hooks/grilling_check.py` | file |

### Generation rules

- Parameterized files are rendered with minijinja; static assets are written byte-identical to the embedded resources.
- Init never deletes or wholesale-overwrites existing content. When a plan path is occupied it merges best-effort:
  - content already equal to the generated content (ignoring trailing whitespace) → `unchanged`;
  - `AGENTS.md` → the generated content is placed in a managed block, appended once and refreshed in place on later runs (`update`); a newly created `AGENTS.md` consists of that block. An `AGENTS.md` without a block that equals the raw generated content is wrapped in a block; one without a block that already contains the harness workflow rules but differs (for example an edited file from hnm before 0.2.0) is `skip`;
  - a Markdown file whose markers are not exactly one well-formed begin/end pair → `skip`;
  - `CLAUDE.md` as a regular file → a managed block containing `@AGENTS.md` is added unless a line `@AGENTS.md` already exists; if it is the same file as `AGENTS.md`, it is `unchanged`;
  - `.claude/settings.json` and `.codex/hooks.json` → JSON objects are deep-merged, missing array items are appended, and existing scalar values win; a type mismatch where the harness needs an object or array (for example `"hooks": null`) → `skip`;
  - `.claude/skills` as a real directory → each harness skill is linked inside it as `.claude/skills/<skill>` → `../../.agents/skills/<skill>`;
  - anything else that differs (other files, invalid JSON, symlinks pointing elsewhere, wrong file types) → left untouched and reported as `skip`.
- Re-running init on an already-initialized directory reports only `unchanged` / `skip-link` and writes nothing.
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

### Clarification reminder

The harness registers an advisory `UserPromptSubmit` hook in both runtime
configurations. Each user turn receives an English v3b reminder, explicitly scoped
to active feature-dev clarification. It respects settled decisions and delegated
authority, asks only material questions requiring user input, and requests a short
contract confirmation before writing PRD, ADR, and spec. Existing PRDs and missing
transcripts do not suppress it. The hook only adds context and has no blocking or
persistent side effects; malformed input and unrelated events produce no output.

## System-wide constraints

- Language: Rust, Cargo, edition `2024`.
- CLI: clap derive.
- Templates: minijinja; resources embedded at compile time from `templates/`.
- Domain errors: `thiserror`; process edge: `anyhow`.
- Large modules are not allowed; split by responsibility (`cli`, `error`, `stack`, `plan`, `render`, `init`).
- This repository dogfoods the same harness layout for developing `hnm` itself.

## Current implementation status

- Product scope and init contract are specified.
- Implementation delivers `hnm init` with clap, minijinja, embedded templates, best-effort merge, dry-run, and automated tests.

## Binary distribution

- Stable releases use `vX.Y.Z`, matching the Cargo package version.
- Release assets cover `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`. Linux requires glibc 2.35+; macOS requires 11+.
- The installer defaults to the latest stable release and `$HOME/.local/bin`; `HNM_VERSION` pins a tag and `HNM_INSTALL_DIR` selects an absolute destination.
- Installation validates SHA-256 and the executable version before replacing the installed binary. Failure exits nonzero and does not report success or replace an existing binary.
- Installing or upgrading the CLI does not modify shell startup files or generated project harnesses.
- Windows installation requires Windows x64 and PowerShell 5.1 or 7; ZIP assets contain hnm.exe. The default is $HOME/.local/bin. The installer adds it to the current process PATH only and prints persistent PATH instructions.
- Windows init retains relative symbolic links and requires Developer Mode or symlink privileges. Installation itself does not require elevation.
