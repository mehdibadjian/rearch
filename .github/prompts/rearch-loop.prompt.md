---
name: rearch-loop
description: Run an autonomous reverse-engineering and modernization loop iteration
inputs:
  slug:
    description: Target slug in legacy/<slug>
    required: true
  window:
    description: "discover (Window A) or build (Window B)"
    required: true
---

Execute one iteration of the `rearch` loop for slug `{{slug}}` in window `{{window}}`:

1. Check current evidence in `analysis/{{slug}}/EVIDENCE_STATE.md`.
2. Form a targeted inspection hypothesis.
3. Query the REA MCP server tools for the relevant module or symbol.
4. Record new findings as `OBSERVATION` or update `UNKNOWN` items.
5. Append iteration progress to `analysis/{{slug}}/LOOP_PROGRESS.md`.
6. Validate loop exit condition using:
   `cargo run --bin rearch -- loop-step {{slug}} --window {{window}}`
