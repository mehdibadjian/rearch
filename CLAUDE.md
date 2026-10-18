# CLAUDE.md: Guidelines for Claude Code in `rearch`

## Commands

- Build workspace: `cargo build --workspace`
- Run release build: `cargo build --workspace --release`
- Run workspace tests: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format check: `cargo fmt -- --check`
- Bootstrap REA: `cargo run --bin rearch -- bootstrap --mode doctor`
- Proof Pack: `cargo run --bin rearch -- proof-pack <slug>`
- Loop Step: `cargo run --bin rearch -- loop-step <slug> --window <discover|build>`

## Architecture

- `crates/rearch-core`: Domain models, 6 Human Decision Gates, prompt-injection guard.
- `crates/rearch-rea`: REA MCP Client (stdio JSON-RPC) and Chrome CDP manager.
- `crates/rearch-proof`: Clean-run proof engine, test result parser, and HTML report generator.
- `crates/rearch-cli`: Orchestrator CLI (`rearch`), multi-host bootstrapper, and loop runner.
