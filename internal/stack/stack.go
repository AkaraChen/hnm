package stack

import (
	"fmt"
	"strings"
)

// Stack is a command-block preset written into generated AGENTS.md.
type Stack string

const (
	Rust    Stack = "rust"
	Node    Stack = "node"
	Bun     Stack = "bun"
	Go      Stack = "go"
	Python  Stack = "python"
	Generic Stack = "generic"
)

func Parse(s string) (Stack, error) {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "rust":
		return Rust, nil
	case "node":
		return Node, nil
	case "bun":
		return Bun, nil
	case "go":
		return Go, nil
	case "python":
		return Python, nil
	case "generic", "":
		return Generic, nil
	default:
		return "", fmt.Errorf("unknown stack `%s`", s)
	}
}

func (s Stack) String() string {
	if s == "" {
		return string(Generic)
	}
	return string(s)
}

func (s Stack) CommandsBlock() string {
	switch s {
	case Rust:
		return rustBlock
	case Node:
		return nodeBlock
	case Bun:
		return bunBlock
	case Go:
		return goBlock
	case Python:
		return pythonBlock
	default:
		return genericBlock
	}
}

const rustBlock = "Package manager and build tooling are Cargo.\n\n- `cargo build` — compile the workspace/package\n- `cargo test` — run unit and integration tests\n- `cargo run` — run the binary\n- `cargo clippy` — lints (prefer clean clippy before merge)\n- `cargo fmt` — rustfmt"
const nodeBlock = "Package manager is npm (adjust if the repo uses pnpm or yarn).\n\n- `npm test` — run tests\n- `npm run build` — production build\n- `npm run lint` — lint\n- `npm run typecheck` — typecheck when configured"
const bunBlock = "Package manager is Bun.\n\n- `bun test` — run tests\n- `bun run build` — production build\n- `bun run lint` — lint\n- `bun run typecheck` — typecheck when configured"
const goBlock = "Tooling is the Go toolchain.\n\n- `go test ./...` — run tests\n- `go build ./...` — compile packages\n- `go vet ./...` — vet\n- `gofmt -w .` — format"
const pythonBlock = "Prefer the repository's configured environment (uv, poetry, or venv).\n\n- `pytest` — run tests when configured\n- `ruff check .` — lint when configured\n- `ruff format .` — format when configured\n- `mypy` — typecheck when configured"
const genericBlock = "Document the repository's real build, test, lint, and format commands here after the first 质问 or when the stack is known.\n\n- Prefer the project's existing package manager and scripts over inventing new ones."
