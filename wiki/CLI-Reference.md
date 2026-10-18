# CLI Reference Guide

The `rearch` binary provides command-line control over all 9 modernization stages, multi-host bootstrapping, and deterministic loop stepping.

```bash
cargo run --bin rearch -- <SUBCOMMAND> [OPTIONS]
```

---

## 🛠️ Subcommands Summary

| Command | Arguments / Flags | Description |
| :--- | :--- | :--- |
| **`bootstrap`** | `--mode <doctor\|dry-run\|apply>` | Provisions or checks `rea-agents` across all 4 host targets |
| **`intent`** | `<slug>` | Initializes modernization intent (`analysis/<slug>/INTENT.md`) |
| **`preflight`** | `<slug>` | Locks legacy SHA-256 digest & records Gate 1 answers |
| **`assess`** | `<slug>` | Performs initial inventory analysis via REA Law 2 |
| **`map`** | `<slug>` | Maps module topology, routes, and call graphs to `map.json` |
| **`extract-rules`** | `<slug>` | Generates BDD Given/When/Then cards in `rules/*.json` |
| **`review`** | `<slug>` | Validates Gate 2 (reviews disputed rule cards) |
| **`brief`** | `<slug>` | Synthesizes architecture brief (`BRIEF.md`) |
| **`sign`** | `<slug>` | Signs architecture brief (fulfills Gate 3) |
| **`uplift`** | `<slug>` | Executes conservative codebase modernization |
| **`transform`** | `<slug>` | Executes framework or language transformation |
| **`reimagine`** | `<slug>` | Executes clean-room greenfield reconstruction |
| **`verify`** | `<slug>` | Executes clean test verification |
| **`loop-step`** | `<slug> [window] [--window <w>] [--max-iterations <N>]` | Advances one iteration of the bounded state machine |
| **`proof-pack`** | `<slug>` | Executes clean test suite and generates `REPORT.html` |
| **`harden`** | `<slug>` | Generates security hardening patch `harden.patch` |

---

## 📖 Command Details

### `bootstrap`
```bash
cargo run --bin rearch -- bootstrap --mode doctor
cargo run --bin rearch -- bootstrap --mode dry-run
cargo run --bin rearch -- bootstrap --mode apply
```
Executes `npx rea-agents` setup/doctor across `antigravity`, `vscode`, `copilot_cli`, and `claude_code`.

### `intent <slug>`
```bash
cargo run --bin rearch -- intent legacy-api
```
Initializes the target analysis workspace and prompts/creates `INTENT.md`.

### `preflight <slug>`
```bash
cargo run --bin rearch -- preflight legacy-api
```
Validates that `legacy/<slug>` exists, computes its cumulative SHA-256 digest, and ensures the 5 Preflight questions are answered in `analysis/<slug>/preflight.json`.

### `loop-step <slug> [window]`
```bash
# Window A (Discovery)
cargo run --bin rearch -- loop-step legacy-api discover --max-iterations 25

# Window B (Reconstruction)
cargo run --bin rearch -- loop-step legacy-api build --max-iterations 50
```
Verifies preconditions, checks for 2-turn stalls, evaluates exit criteria, and regenerates `analysis/<slug>/REPORT.html`.

### `proof-pack <slug>`
```bash
cargo run --bin rearch -- proof-pack legacy-api
```
Purges dirty build caches, invokes the test runner, parses machine-readable results, outputs `analysis/<slug>/proof/verdict.json`, and renders `REPORT.html`.
