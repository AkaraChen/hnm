<p align="center">
  <img src="docs/assets/hnm-cover.jpg" alt="hnm — illustration of three document folders connected into one system" width="960">
</p>

<h1 align="center">hnm</h1>

<p align="center">
  Install a documentation workflow for your coding agents, in one command.
</p>

<p align="center">
  <a href="https://github.com/AkaraChen/hnm/actions/workflows/release.yml"><img src="https://github.com/AkaraChen/hnm/actions/workflows/release.yml/badge.svg" alt="Release workflow status"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#what-gets-written">File layout</a> ·
  <a href="docs/spec.md">Specification</a> ·
  <a href="https://github.com/AkaraChen/hnm/releases/latest">Releases</a>
</p>

Give Claude and Codex a shared place for requirements, decisions, and project rules.
`hnm init` installs the docs, skills, and commit review hooks into your repository.

## A workflow your agents can follow

- **Clarify before building.** The `$feature-dev` skill walks through product and technical questions one at a time (质问), then records the plan before implementation.
- **Keep decisions in the repo.** PRDs describe what to build, ADRs explain the choices, and `docs/spec.md` holds shared contracts.
- **Review docs before committing.** Shared `AGENTS.md` rules, the `$git-commit` skill, and Claude/Codex hooks connect documentation review to the commit workflow.

Skills and hooks take effect in agent runtimes that load the installed configuration.
`hnm` sets up the workflow files; it does not generate application code or initialize Git.

## Quick start

### 1. Install hnm

**macOS / Linux**

```sh
curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh | sh
```

Requires macOS 11+ or Linux with glibc 2.35+, on Intel/x86_64 or ARM64.
Add `~/.local/bin` to your PATH if needed:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

**Windows · PowerShell**

```powershell
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
```

Requires Windows x64 and PowerShell 5.1 or 7. The installer adds hnm to the current
session PATH and prints instructions for future terminals. To run `hnm init`,
enable Developer Mode or use an account with symlink privileges.

No Rust toolchain is needed. Installers check SHA-256 and the binary version before
replacing an existing installation. Rerun to upgrade the CLI; project harnesses are untouched.

[Installation details](docs/installation.md) cover dependencies, platform limits,
script inspection, version pinning, custom paths, and source builds.

### 2. Preview, then initialize

From your project directory:

```sh
hnm init --dry-run
hnm init
```

Existing files are merged best-effort instead of overwritten: `AGENTS.md` and a regular
`CLAUDE.md` get an hnm-managed block, JSON hook configs are deep-merged, and a real
`.claude/skills` directory gets per-skill links. Anything that cannot be merged safely is left
untouched and reported as `skip`. Review the printed summary, then open `AGENTS.md` and `docs/spec.md` to adapt the generated guidance to your project.

## Usage

```sh
# Set the target directory, project name, and command preset
hnm init ./my-app --name my-app --stack rust

# Preview the same setup without writing files
hnm init ./my-app --name my-app --stack rust --dry-run
```

Re-running `hnm init` is safe: it refreshes the managed blocks and reports unchanged paths. Without `--name`, the name comes from the target directory.

### Stack presets

Presets fill the **Commands** section of `AGENTS.md`; they do not install dependencies.

| `--stack` | Commands section |
| --- | --- |
| `generic` (default) | Placeholder notes |
| `rust` | cargo build/test/clippy/fmt |
| `node` | npm scripts |
| `bun` | bun scripts |
| `go` | go test/build/vet |
| `python` | pytest/ruff/mypy when configured |

## What gets written

```text
project/
├── AGENTS.md                 Workflow rules + stack commands
├── CLAUDE.md → AGENTS.md
├── docs/
│   ├── prd/                  Product requirements
│   ├── adr/                  Architecture decisions
│   └── spec.md               Shared contracts
├── .agents/skills/
│   ├── feature-dev/          Clarification before implementation
│   └── git-commit/           Conventional commits from the diff
├── .claude/
│   ├── skills → ../.agents/skills
│   └── settings.json         Commit doc-review hook
└── .codex/
    ├── hooks.json
    └── hooks/spec_doc_review.py
```

The skills include `SKILL.md` and `agents/openai.yaml`; empty PRD/ADR directories
include `.gitkeep`. See the [complete generated layout](docs/spec.md#generated-harness-layout).

## Project docs

- [Specification](docs/spec.md) — CLI behavior, generation rules, and platform contracts.
- [Harness requirements](docs/prd/harness-init.md) — scope and intended workflow.
- [Implementation decisions](docs/adr/clap-minijinja-embedded-templates.md) — CLI, templates, and embedded resources.
- [Installation](docs/installation.md) · [Release process](docs/releasing.md)

## Develop

With a Rust toolchain supporting edition 2024, clone this repository and run:

```sh
cargo test
cargo run -- init --help
```

See [AGENTS.md](AGENTS.md) for the development workflow.

<sub>Cover: AI-generated conceptual illustration, not a product screenshot. <a href="docs/readme-design.md">Design notes</a>.</sub>
