# Autonomous Specialist Agents in `rearch`

This workspace defines four specialist agent roles coordinated by the `rearch` orchestrator:

1. **`architecture-critic`** (`.agents/agents/architecture-critic.md`): Read-only reviewer evaluating modernized architecture and steering briefs.
2. **`rule-extractor`** (`.agents/agents/rule-extractor.md`): Author of BDD Given/When/Then rule cards with REA citations.
3. **`rule-verifier`** (`.agents/agents/rule-verifier.md`): Independent auditor cross-checking extracted rules against executable evidence.
4. **`code-simplifier`** (`.agents/agents/code-simplifier.md`): Refactoring specialist cleaning modernized modules while maintaining test parity.

## Operational Constraints

- **Deterministic Verification:** Every loop iteration is verified with `cargo run --bin rearch -- loop-step <slug> --window <discover|build>`.
- **Legacy Immutability:** Files in `legacy/<slug>` are verified by SHA-256 digest and must never be modified.
- **Human Gates:** Gates 1-6 block unauthorized state transitions until human sign-offs are in place.
