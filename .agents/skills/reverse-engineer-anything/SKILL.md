---
name: reverse-engineer-anything
description: Reverse engineer any software artifact (web service, frontend bundle, binary, container) using smallest useful inventory first, hypothesis-driven inspection, and four epistemic tiers.
---

# Reverse Engineer Anything (REA)

## Operational Principles

1. **Smallest Useful Inventory First:**
   Always inspect high-level topology (`map.json`) before decompiling deep functions or dumping bundle ASTs.
2. **Hypothesis-Driven Deep Inspection:**
   Formulate a specific question before running deep inspection tools.
3. **Four Epistemic Tiers:**
   - `OBSERVATION`: Direct tool output with line/offset citations.
   - `DERIVATION`: Deterministic transformations (e.g. types synthesized from observed payloads).
   - `INFERENCE`: Architectural hypotheses.
   - `UNKNOWN`: Unverified questions.
4. **Executable Truth Over Comments:**
   Trust bytecode, traces, and execution over human-written comments when discrepancies occur.
