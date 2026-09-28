# Installation

[← Back to README](../README.md) · **English** · [简体中文](installation.zh-CN.md)

Install the latest release (no Rust toolchain required):

```sh
curl -fsSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh | sh
```

Supports macOS 11+ (Intel / Apple Silicon) and Linux with glibc 2.35+
(x86_64 / ARM64; for example Ubuntu 22.04+). Alpine/musl is not supported
by the Unix installer. CI tests on Ubuntu 22.04, macOS 14 ARM and macOS 15
Intel; the minimum macOS version is a build target, not a tested OS.

The script verifies SHA-256 and `hnm --version`, then installs to `~/.local/bin`.
Add `export PATH="$HOME/.local/bin:$PATH"` to your shell configuration if needed.
Rerun the same command to upgrade; existing project harnesses are untouched.
Requires `curl`, `tar`, and `sha256sum` or `shasum`. No sudo is used.

## Windows (PowerShell)

```powershell
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
```

Supports Windows x64 with PowerShell 5.1 or 7. No Rust, Git Bash, or administrator
access is needed to install. SHA-256 and the binary version are checked before
replacing an existing installation. Defaults to `$HOME/.local/bin`, adds it to
the current session PATH, and prints instructions for future terminals.
`HNM_VERSION` and `HNM_INSTALL_DIR` environment variables are also supported.
Windows ARM64 native packages are not provided.

`hnm init` creates symbolic links: enable Windows Developer Mode or run with
symlink privileges. The installer does not change system policies.

## Options

To inspect the script, select a version, or customize the destination:

```sh
curl -fSL https://github.com/AkaraChen/hnm/releases/latest/download/install.sh -o install.sh
# Review install.sh before running it.
HNM_VERSION=v0.2.0 HNM_INSTALL_DIR="$HOME/.local/bin" sh install.sh
hnm --version
```

`HNM_VERSION` accepts `latest` (default) or a stable `vX.Y.Z` tag.
`HNM_INSTALL_DIR` must be absolute; on Windows it must be a local drive path.
PowerShell example:

```powershell
$env:HNM_VERSION = 'v0.2.0'
$env:HNM_INSTALL_DIR = "$HOME\.local\bin"
irm https://github.com/AkaraChen/hnm/releases/latest/download/install.ps1 | iex
hnm --version
```

A missing release/asset, unsupported platform, checksum failure, or incompatible
binary exits nonzero and leaves the existing executable intact. Checksums protect
against corruption; downloads trust the repository and GitHub HTTPS.

The Unix pipeline can hide the initial script download's failure exit code.
For automation requiring strict failure detection, download successfully before
executing, as in the example above.

## Build from source

For other systems, install a Rust toolchain with edition 2024 support, clone this repository, and build from source:

```sh
git clone https://github.com/AkaraChen/hnm.git
cd hnm
cargo install --path . --locked
```

Ensure Cargo's bin directory (usually `~/.cargo/bin`) is on PATH.
After building, enter your own project root and run `hnm init`.

## Troubleshooting

- **`hnm` not found:** check that installation succeeded and the install directory
  is on PATH. On Unix, use `export PATH="$HOME/.local/bin:$PATH"`; on Windows,
  follow the installer's instructions.
- **Windows symlink creation fails:** enable Developer Mode or obtain symlink
  privileges, then rerun init.
- **Changing name or stack has no effect:** rerunning refreshes the managed block in
  `AGENTS.md`, but an existing `docs/spec.md` is never overwritten. Inspect with
  `--dry-run`, then edit or remove that file yourself and rerun.
- **Hooks do not run:** confirm your agent supports and loads project configuration.
  The review hook needs Python 3; the bundled Codex path is `/usr/bin/python3`,
  which you may need to adapt to your OS.

See the [English README](../README.md) for init examples, options, and next steps.
