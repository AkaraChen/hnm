# hnm

Install a complete **agent documentation harness** into any project:

- `docs/prd/`, `docs/adr/`, `docs/spec.md`
- `AGENTS.md` + `CLAUDE.md` symlink
- `$feature-dev` skill (one-question 质问 before implementation)
- `$git-commit` skill (conventional commits from the diff)
- Claude/Codex skill wiring and pre-commit documentation review gate

## Install

Install the latest release (no Rust toolchain required):

```sh
(set -eu; f=$(mktemp); trap 'rm -f "$f"' EXIT; curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh -o "$f"; sh "$f")
```

Supports macOS 11+ (Intel / Apple Silicon) and Linux with glibc 2.35+
(x86_64 / ARM64; for example Ubuntu 22.04+). Windows and Alpine/musl are not
supported by this installer. CI tests on Ubuntu 22.04, macOS 14 ARM and macOS 15
Intel; the minimum macOS version is a build target, not a tested OS.

The script verifies SHA-256 and `hnm --version`, then installs to `~/.local/bin`.
Add `export PATH="$HOME/.local/bin:$PATH"` to your shell configuration if needed.
Rerun the same command to upgrade; existing project harnesses are untouched.
Requires `curl`, `tar`, and `sha256sum` or `shasum`. No sudo is used.

To inspect the script, select a version, or customize the destination:

```sh
curl -fSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh -o install.sh
# Review install.sh before running it.
HNM_VERSION=v0.1.0 HNM_INSTALL_DIR="$HOME/.local/bin" sh install.sh
hnm --version
```

A missing release/asset, unsupported platform, checksum failure, or incompatible
binary exits nonzero and leaves the existing executable intact. Checksums protect
against corruption; downloads trust the repository and GitHub HTTPS.

For other systems, clone this repository and build from source:

```sh
cargo install --path . --locked
```

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

## Release

1. Update `Cargo.toml` and `Cargo.lock` to the next stable version; merge the PR.
2. Push the matching tag, for example `git tag v0.1.0 && git push origin v0.1.0`.
3. The Release workflow tests and packages all four native targets, uploads the
   archives, per-file SHA-256 checksums, and installer to a draft, then publishes
   it only after every build passes. A final matrix installs from the public
   Release and runs `--version` and `init`.

Only the publish job needs `contents: write` through `GITHUB_TOKEN`; builds and
PR checks are read-only. No personal token or external service is required.
Tags must exactly match the Cargo version. Failed draft uploads can be rerun;
published assets are never overwritten (fixes require a new version).
Each archive is named `hnm-vX.Y.Z-<rust-target>.tar.gz`, contains `hnm`, and has a
matching `.sha256` file. `install.sh` and `install.sh.sha256` are also assets.
