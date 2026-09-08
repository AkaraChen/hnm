use std::fmt;
use std::str::FromStr;

use clap::ValueEnum;

/// Command-block preset written into generated `AGENTS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum Stack {
    Rust,
    Node,
    Bun,
    Go,
    Python,
    #[default]
    Generic,
}

impl Stack {
    pub fn commands_block(self) -> &'static str {
        match self {
            Self::Rust => {
                "\
Package manager and build tooling are Cargo.

- `cargo build` — compile the workspace/package
- `cargo test` — run unit and integration tests
- `cargo run` — run the binary
- `cargo clippy` — lints (prefer clean clippy before merge)
- `cargo fmt` — rustfmt"
            }
            Self::Node => {
                "\
Package manager is npm (adjust if the repo uses pnpm or yarn).

- `npm test` — run tests
- `npm run build` — production build
- `npm run lint` — lint
- `npm run typecheck` — typecheck when configured"
            }
            Self::Bun => {
                "\
Package manager is Bun.

- `bun test` — run tests
- `bun run build` — production build
- `bun run lint` — lint
- `bun run typecheck` — typecheck when configured"
            }
            Self::Go => {
                "\
Tooling is the Go toolchain.

- `go test ./...` — run tests
- `go build ./...` — compile packages
- `go vet ./...` — vet
- `gofmt -w .` — format"
            }
            Self::Python => {
                "\
Prefer the repository's configured environment (uv, poetry, or venv).

- `pytest` — run tests when configured
- `ruff check .` — lint when configured
- `ruff format .` — format when configured
- `mypy` — typecheck when configured"
            }
            Self::Generic => {
                "\
Document the repository's real build, test, lint, and format commands here after the first 质问 or when the stack is known.

- Prefer the project's existing package manager and scripts over inventing new ones."
            }
        }
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Rust => "rust",
            Self::Node => "node",
            Self::Bun => "bun",
            Self::Go => "go",
            Self::Python => "python",
            Self::Generic => "generic",
        };
        f.write_str(s)
    }
}

impl FromStr for Stack {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "rust" => Ok(Self::Rust),
            "node" => Ok(Self::Node),
            "bun" => Ok(Self::Bun),
            "go" => Ok(Self::Go),
            "python" => Ok(Self::Python),
            "generic" => Ok(Self::Generic),
            other => Err(format!("unknown stack `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_block_mentions_cargo() {
        assert!(Stack::Rust.commands_block().contains("cargo test"));
    }

    #[test]
    fn parse_stack_names() {
        assert_eq!("bun".parse::<Stack>().unwrap(), Stack::Bun);
        assert!("nope".parse::<Stack>().is_err());
    }
}
