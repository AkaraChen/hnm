# hnm

Install a shared documentation workflow for Claude and Codex in your project.

**English** · [简体中文](README.zh-CN.md)

## Quick start

**macOS / Linux** — replace the `cd` path with your existing project root.

```sh
curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
cd /path/to/your-project
hnm init
```

**Windows · PowerShell** — replace the `cd` path with your existing project root.

```powershell
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
cd C:\path\to\your-project
hnm init
```

## Requirements and installation

No Rust toolchain is needed for release binaries. macOS 11+ and Linux with glibc
2.35+ support x86_64 and ARM64; the Unix installer needs `curl`, `tar`, and
`sha256sum` or `shasum`. Alpine/musl is not supported by that installer.
Windows requires x64 and PowerShell 5.1 or 7; **enable Developer Mode or use
symlink privileges before running `hnm init`**.

Installers check SHA-256 and the executable version, then install to
`~/.local/bin`. The Unix `export` above sets PATH for this terminal; add it to
your shell configuration for future terminals. The Windows installer sets the
current session PATH and prints persistent PATH instructions. Rerun the installer
to upgrade the CLI; existing project harnesses are untouched.

[Installation guide](docs/installation.md): inspect the script, pin a version,
choose a destination, build from source, and troubleshoot.

## After init

1. Read the printed summary. Existing files are merged rather than overwritten
   (`update`); only paths that cannot be merged safely are reported as `skip`,
   so inspect those to ensure the workflow is complete.
2. Open `AGENTS.md` and `docs/spec.md`; set your project name, real commands, and
   shared rules. Plain `hnm init` uses the `generic` command preset and embeds
   the fallback name `project` for the default `.` path. Use `hnm init --name my-app
   --stack rust` on the first run to set both explicitly.
3. Open the project in an agent runtime that loads project skills and hooks.
   Ask it to use `$feature-dev` for the next feature: clarify requirements,
   record `docs/prd/`, `docs/adr/`, and `docs/spec.md`, then implement. Use
   `$git-commit` when ready to review the diff and commit.

`hnm` writes workflow files; it does not generate application code, install
application dependencies, or run `git init`. Generated templates remain in their
bundled language; switching README language does not change generated files.

## Usage and configuration

```sh
# Inspect all options
hnm init --help

# Preview without writing (even if the target directory does not exist)
hnm init ./my-app --name my-app --stack rust --dry-run

# Initialize a specific directory (created if missing)
hnm init ./my-app --name my-app --stack rust
```

| Argument | Default | Effect |
| --- | --- | --- |
| `[PATH]` | `.` | Target directory |
| `--name`, `-n` | Final path component, or `project` for `.` / `..` | Project name in templates |
| `--stack`, `-s` | `generic` | Commands preset below |
| `--dry-run` | Off | Print the plan without writing |

Configuration is through CLI arguments; there is no hnm configuration file.
Installer environment variables `HNM_VERSION` and `HNM_INSTALL_DIR` are described
in the [installation guide](docs/installation.md#options).

Init never deletes or wholesale-replaces existing content; it merges best-effort:
`AGENTS.md` gets an hnm-managed block (`<!-- hnm:begin -->` … `<!-- hnm:end -->`)
that is refreshed on later runs, a regular `CLAUDE.md` gains an `@AGENTS.md` import,
`.claude/settings.json` and `.codex/hooks.json` are deep-merged (your values win),
and a real `.claude/skills` directory gets per-skill links. Other differing files,
invalid JSON, and foreign symlinks are left untouched and reported as `skip`, with a
closing note. Rerunning is safe. Failures exit nonzero and may leave files already
written; fix the error and rerun. A successful exit can still include skipped paths.

### Stack presets

Presets fill the **Commands** section of `AGENTS.md`; they do not detect your
stack or install dependencies. Edit the generated commands to match your project.

| `--stack` | Commands section |
| --- | --- |
| `generic` (default) | Placeholder notes |
| `rust` | cargo build/test/run/clippy/fmt |
| `node` | npm test/build/lint/typecheck scripts |
| `bun` | bun test/build/lint/typecheck scripts |
| `go` | go test/build/vet and gofmt |
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
│   └── settings.json         Documentation review and clarification hooks
└── .codex/
    ├── hooks.json
    └── hooks/
        ├── spec_doc_review.py
        └── grilling_check.py
```

The skills include `SKILL.md` and `agents/openai.yaml`; empty PRD/ADR directories
include `.gitkeep`. See the [complete generated layout](docs/spec.md#generated-harness-layout).

## A workflow your agents can follow

<p align="center">
  <img src="docs/assets/hnm-cover.jpg" alt="hnm — three document folders connected into one system" width="960">
</p>

- **Clarify before building.** `$feature-dev` asks product and technical questions
  one at a time (质问) and records the plan before implementation.
- **Keep decisions in the repo.** PRDs describe what to build, ADRs explain choices,
  and `docs/spec.md` holds shared contracts.
- **Review docs before committing.** `AGENTS.md`, `$git-commit`, and Claude/Codex
  hooks connect documentation review to commits in runtimes that load them.
  The English grilling reminder supplies context on each user turn, scoped to
  active feature-dev clarification. It respects settled decisions and delegated
  authority; it is advisory rather than a permission gate.
  The installed hooks require Python 3 (`/usr/bin/python3` in the bundled
  Codex configuration); adapt hook commands to your runtime and OS.

[Release workflow](https://github.com/AkaraChen/hnm/actions/workflows/release.yml) ·
[Releases](https://github.com/AkaraChen/hnm/releases/latest)

## Project docs

- [Installation](docs/installation.md) — user guide in English and Chinese.
- [Specification](docs/spec.md) — CLI behavior, layout, and platform contracts (English).
- [Harness requirements](docs/prd/harness-init.md) — scope and workflow (Chinese).
- [Implementation decisions](docs/adr/clap-minijinja-embedded-templates.md) — CLI and templates (Chinese).
- [Release process](docs/releasing.md) — maintainer guide (English).

## Develop

With a Rust toolchain supporting edition 2024:

```sh
git clone https://github.com/AkaraChen/hnm.git
cd hnm
cargo test
cargo run -- init --help
```

See [AGENTS.md](AGENTS.md) for the development workflow.

<sub>Cover: AI-generated conceptual illustration. <a href="docs/readme-design.md">Design notes</a>.</sub>
