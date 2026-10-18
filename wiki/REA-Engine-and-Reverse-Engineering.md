# REA Engine & Reverse Engineering

`rearch` embeds **REA (`morluto/rea`)** as its sole inspection and decompilation authority. Built as a native Rust client in `crates/rearch-rea`, it interacts with the `rea-agents` Model Context Protocol (MCP) server over stdio JSON-RPC.

---

## 📜 The Four REA Laws

All inspection turns must strictly obey REA's core operational discipline:

### Law 1: Session Reuse by Digest
Before opening a new analysis workspace, disassembler session, or browser instance, `rearch-rea` checks `analysis/<slug>/rea_sessions.json`. If an active session already exists for the legacy artifact's cumulative SHA-256 digest, the existing session is reused. This prevents duplicate memory-heavy Ghidra/JADX/Chrome sessions.

### Law 2: Smallest Useful Inventory First
Deep decompilation, function disassembly, and full-bundle AST dumps are blocked until the target's top-level inventory (`analysis/<slug>/map.json`) is populated. 
- *Web services:* Check route lists and HTTP methods before payload parsing.
- *Websites:* Enumerate scripts, stylesheets, and custom elements before tracing ASTs.
- *Binaries:* Enumerate header formats, sections, exports, and imported symbols before decompiling function bodies.

If an agent attempts to invoke `decompile`, `disassemble`, or `ast_dump` before `map.json` exists, the MCP client returns an error:
```text
REA Law 2 Violation: Smallest Useful Inventory First. Populate map.json before deep inspection.
```

### Law 3: Hypothesis-Driven Inspection
Inspection is never exhaustive or random. Each iteration must target a specific `UNKNOWN` row in `analysis/<slug>/EVIDENCE_STATE.md`.

### Law 4: Untrusted Legacy Content Guard
Legacy artifacts (JavaScript bundles, binary strings, database seeds, error messages) can contain malicious adversarial prompts or prompt-injection attempts intended to hijack LLM behavior. Every raw payload returned by the REA MCP bridge is passed through `rearch_core::guard::scan_untrusted_output`. Detected injection attempts are sanitized, blocked, and appended to `analysis/<slug>/security_findings.jsonl`.

---

## 🔍 Target Routing & Capabilities

| Target Category | Setup Tooling | Smallest Inventory First | Deep Inspection |
| :--- | :--- | :--- | :--- |
| **Web Service** (`.har` or URL) | Mount `.har` in `legacy/<slug>` or attach CDP | REA network waterfall & route table | Request/response schema extraction, auth header inspection |
| **Website / Frontend** (Live DOM / Bundle) | Launch isolated Chrome CDP (`127.0.0.1:9222`) | DOM tree summary, script tags, CSS token map | AST dumping, runtime variable state inspection |
| **Native Binary** (ELF / PE / Mach-O / Archive) | Symlink into `legacy/<slug>`, compute SHA-256 | `goblin` header & export table inspection | Ghidra / IDA / Hopper function disassembly |
| **Android / Java** (`.apk` / `.jar`) | Validate package checksum | Manifest & package hierarchy | JADX decompiled class and method extraction |
| **.NET / WebAssembly** (`.dll` / `.wasm`) | Symlink and lock | Metadata tables, exported functions | ILSpy / wasm2wat decompilation |

---

## 🌐 Chrome CDP Integration (`crates/rearch-rea/src/cdp.rs`)

For client-rendered single page applications (SPAs) and dynamic websites, `rearch-rea` manages an isolated headless Chrome instance:
- Bound strictly to `127.0.0.1:9222`.
- Ephemeral user-data directory in `target/cdp_profiles/<slug>`.
- Allows DOM querying, JavaScript execution evaluation, and network capture.
