# Multi-Host Setup & Bootstrapping

`rearch` is built from the ground up for seamless operation across major AI coding hosts: **Google Antigravity**, **VS Code Copilot**, and **Claude Code**.

---

## 🛠️ Bootstrapper CLI (`rearch bootstrap`)

The harness includes a built-in multi-host bootstrapper in [`crates/rearch-cli/src/bootstrap.rs`](../crates/rearch-cli/src/bootstrap.rs) that validates and configures the environment for all 4 client IDs (`antigravity`, `vscode`, `copilot_cli`, `claude_code`).

### Bootstrapper Commands

1. **Doctor Mode (Check environment prerequisites):**
   ```bash
   cargo run --bin rearch -- bootstrap --mode doctor
   ```
   Verifies Node.js, `npx`, `rea-agents`, Chrome CDP availability, and required binary tools (e.g. `goblin`, `quick-xml`).

2. **Dry-Run Mode (Preview setup changes):**
   ```bash
   cargo run --bin rearch -- bootstrap --mode dry-run
   ```

3. **Apply Mode (Configure MCP and settings):**
   ```bash
   cargo run --bin rearch -- bootstrap --mode apply
   ```

---

## 💻 Host Integrations

### 1. Google Antigravity
- **Configuration:** [`.antigravity/config.json`](../.antigravity/config.json)
- **Specialist Subagent Mapping:** Binds `architecture-critic`, `rule-extractor`, `rule-verifier`, and `code-simplifier`.
- **Loop Orchestration:** Antigravity triggers `rearch loop-step <slug> <discover|build>` to verify deterministic state transitions at each turn.

### 2. VS Code Copilot
- **MCP Registration:** [`.vscode/mcp.json`](../.vscode/mcp.json) configures the `rea` MCP server for GitHub Copilot Agent mode.
- **Tasks:** [`.vscode/tasks.json`](../.vscode/tasks.json) exposes build, test, and step tasks.
- **Shared Instructions:** [`.github/copilot-instructions.md`](../.github/copilot-instructions.md) enforces REA laws and gate validation.
- **Chat Command:** [`.github/prompts/rearch-loop.prompt.md`](../.github/prompts/rearch-loop.prompt.md) enables the `/rearch-loop` workflow in VS Code chat.

### 3. Claude Code
- **Settings:** [`.claude/settings.json`](../.claude/settings.json) registers the `rea` stdio MCP server and disables non-essential telemetry.
- **Custom Slash Command:** [`.claude/commands/rearch-loop.md`](../.claude/commands/rearch-loop.md) defines `/rearch-loop <slug> <discover|build>` to orchestrate iterative modernization loops.
