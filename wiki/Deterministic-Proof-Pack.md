# Deterministic Proof Pack

The proof pack engine ([`crates/rearch-proof`](../crates/rearch-proof)) deterministically validates that modernized implementations maintain parity with legacy behavioral rules and passing test suites.

---

## 🧼 Clean-Run Guarantees

AI coding assistants frequently suffer from test cache false positives. `rearch proof-pack` guarantees deterministic results by enforcing the following clean-run protocol:

1. **Cache Purge:** Deletes all cached build outputs and test caches (`target/`, `.vitest-cache/`, `build/`) before running test commands.
2. **Machine-Readable Parsers:** Test pass/fail counts are never extracted from conversational summaries or stdout echoes. The verifier reads directly from:
   - Cargo test output (`test result: ok. X passed; 0 failed`)
   - JUnit XML files (`<testsuites tests="X" failures="0">`) parsed via `quick-xml`
   - Vitest / Jest JSON reports (`{ "numPassedTests": X, "numFailedTests": 0 }`)
3. **The Zero-Test Rule:** If the test suite finishes with `0` executed tests (`total == 0`), the run is immediately marked **`NOT PROVEN`**. Parity cannot be proven with empty test suites.
4. **SHA-256 Legacy Verification:** Ensures `legacy/<slug>` was not altered to make tests pass.
5. **Gate 4 Divergence Check:** Any detected test divergence must match an entry in `analysis/<slug>/accepted_divergences.json`.

---

## ⚖️ Proof Verdicts

The verification engine produces a structured verdict in `analysis/<slug>/proof/verdict.json`:

| Verdict | Meaning | Exit Loop? |
| :--- | :--- | :--- |
| **`PROVEN`** | All tests pass, test count > 0, 0 unaccepted divergences, all rule cards confirmed. | ✅ Yes (Exits Window B) |
| **`PARTLY PROVEN`** | Tests pass with accepted human divergences (`Gate 4`) or partial test coverage accepted by human sign-off. | ✅ Yes (Exits Window B) |
| **`NOT PROVEN`** | Tests failed, compilation error, 0 tests ran, legacy mutated, or unreviewed divergence exists. | ❌ No (Loop continues) |

---

## 📊 Self-Contained HTML Report (`analysis/<slug>/REPORT.html`)

Using the Askama template engine ([`templates/report.html`](../crates/rearch-proof/templates/report.html)), `rearch` generates a self-contained HTML dashboard:
- **System Metadata:** Target slug, timestamp, legacy SHA-256 digest.
- **Decision Gate Badges:** Visual status (LOCKED, PENDING, APPROVED) for Gates 1 through 6.
- **Proof Pack Status:** Test counts (passed, failed, total) and overall verdict.
- **Rule Cards Table:** All extracted BDD specifications, their epistemic tier, and verification status.

The HTML report is refreshed after every single CLI stage and loop iteration.
