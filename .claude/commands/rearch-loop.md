# /rearch-loop <slug> <discover|build>

Autonomous REA & Code-Modernization Loop command for Claude Code.

## Workflow

1. Validate inputs:
   - `<slug>`: The target service or binary identifier in `legacy/<slug>`.
   - `<discover|build>`: Loop Window A (discovery & BDD rule extraction) or Window B (pinning tests & synthesis).
2. Invoke REA MCP tools:
   - For Window A (`discover`): Inspect target, update `analysis/<slug>/map.json` and `analysis/<slug>/rules/*.json`. Update `analysis/<slug>/EVIDENCE_STATE.md`.
   - For Window B (`build`): Implement modern module in `modernized/<slug>`, run tests.
3. Record progress:
   - Append concise outcome signature to `analysis/<slug>/LOOP_PROGRESS.md`.
4. Verification & Termination:
   - Run verification step:
     ```bash
     cargo run --bin rearch -- loop-step $ARG_1 --window $ARG_2
     ```
   - If the command returns code 0 (`GOAL MET`), the loop exits successfully.
   - If non-zero (`CONTINUE_LOOP`), proceed to next iteration with refined hypothesis.
