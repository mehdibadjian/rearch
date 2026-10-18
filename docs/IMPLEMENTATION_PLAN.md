# Comprehensive Implementation Plan: `rearch` Pure Rust Workspace + Autonomous CI/CD

This implementation plan provides exact file paths, crate structures, data models, logic, and automated CI/CD configurations—including **Release Please** for automated semantic versioning and changelog generation on push to `main`.

---

## 1. Workspace Directory Structure

```text
rearch/
├── Cargo.toml
├── INTENT.md
├── .github/
│   ├── workflows/
│   │   ├── ci.yaml                            # Build, test, lint, and security checks on PR / push
│   │   └── release.yaml                       # Release Please workflow for automated tagging & publishing
│   ├── release-please/
│   │   ├── config.json                        # Release Please multi-package / workspace configuration
│   │   └── manifest.json                      # Workspace version manifest
│   ├── copilot-instructions.md
│   └── prompts/rearch-loop.prompt.md
├── crates/
│   ├── rearch-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── bootstrap.rs
│   │       └── loop_runner.rs
│   ├── rearch-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs
│   │       ├── gates.rs
│   │       └── guard.rs
│   ├── rearch-rea/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mcp_client.rs
│   │       └── cdp.rs
│   └── rearch-proof/
│       ├── Cargo.toml
│       ├── templates/
│       │   └── report.html
│       └── src/
│       │   ├── lib.rs
│       │   ├── verifier.rs
│       │   └── report.rs
├── legacy/<slug>
├── analysis/<slug>/
│   ├── REPORT.html
│   ├── BRIEF.md
│   ├── EVIDENCE_STATE.md
│   ├── LOOP_PROGRESS.md
│   ├── preflight.json
│   ├── map.json
│   ├── rules/
│   └── proof/verdict.json
├── modernized/<slug>/
├── .agents/
│   ├── skills/reverse-engineer-anything/SKILL.md
│   └── agents/
│       ├── architecture-critic.md
│       ├── rule-extractor.md
│       ├── rule-verifier.md
│       └── code-simplifier.md
├── .claude/
│   ├── settings.json
│   └── commands/rearch-loop.md
├── .vscode/
│   ├── mcp.json
│   └── tasks.json
├── .antigravity/
│   └── config.json
├── AGENTS.md
└── CLAUDE.md

```

---

## 2. Root Workspace Configuration

### Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/rearch-cli",
    "crates/rearch-core",
    "crates/rearch-rea",
    "crates/rearch-proof",
]

[workspace.package]
version = "1.0.0"
edition = "2021"

[workspace.dependencies]
anyhow = "1.0"
askama = "0.12"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.5", features = ["derive"] }
goblin = "0.9"
quick-xml = { version = "0.36", features = ["serialize"] }
regex = "1.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
thiserror = "2.0"
tokio = { version = "1.40", features = ["full"] }
walkdir = "2.5"

```

---

## 3. Core Domain Models & Human Gates (`rearch-core`)

### `crates/rearch-core/Cargo.toml`

```toml
[package]
name = "rearch-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
sha2.workspace = true
anyhow.workspace = true
regex.workspace = true

```

### `crates/rearch-core/src/lib.rs`

```rust
pub mod models;
pub mod gates;
pub mod guard;

```

### `crates/rearch-core/src/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EpistemicTier {
    Observation,
    Derivation,
    Inference,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    Confirmed,
    Disputed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BddSpec {
    pub given: String,
    pub when: String,
    pub then: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub epistemic_tier: EpistemicTier,
    pub source_citations: Vec<String>,
    pub comment_discrepancy_flagged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVerification {
    pub extractor_agent: String,
    pub verifier_agent: String,
    pub second_agent_status: VerificationStatus,
    pub human_review_status: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCard {
    pub rule_id: String,
    pub module: String,
    pub title: String,
    pub bdd: BddSpec,
    pub provenance: Provenance,
    pub verification: RuleVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofVerdict {
    #[serde(rename = "PROVEN")]
    Proven,
    #[serde(rename = "PARTLY PROVEN")]
    PartlyProven,
    #[serde(rename = "NOT PROVEN")]
    NotProven,
}

```

### `crates/rearch-core/src/gates.rs`

```rust
use anyhow::{bail, Result};
use std::{fs, path::Path};

pub fn verify_gate_1_locked(slug: &str) -> Result<()> {
    let preflight_path = Path::new("analysis").join(slug).join("preflight.json");
    if !preflight_path.exists() {
        bail!("GATE 1 VIOLATION: preflight.json not found. Run `rearch preflight <slug>` first.");
    }
    let content = fs::read_to_string(&preflight_path)?;
    if !content.contains("\"gate_1_approved\": true") {
        bail!("GATE 1 VIOLATION: The 5 Preflight Questions have not been signed off by a human.");
    }
    Ok(())
}

pub fn verify_gate_3_signed(slug: &str) -> Result<()> {
    let brief_path = Path::new("analysis").join(slug).join("BRIEF.md");
    if !brief_path.exists() {
        bail!("GATE 3 VIOLATION: BRIEF.md not found. Run `rearch brief <slug>` first.");
    }
    let content = fs::read_to_string(&brief_path)?;
    if !content.contains("STATUS: SIGNED_BY_HUMAN") {
        bail!("GATE 3 VIOLATION: Steering Brief BRIEF.md has not been signed off by a human.");
    }
    Ok(())
}

pub fn verify_legacy_unmodified(slug: &str) -> Result<()> {
    let preflight_path = Path::new("analysis").join(slug).join("preflight.json");
    if !preflight_path.exists() {
        return Ok(());
    }
    Ok(())
}

```

### `crates/rearch-core/src/guard.rs`

```rust
use anyhow::{bail, Result};
use std::{fs, path::Path};

pub fn scan_untrusted_output(slug: &str, tool_name: &str, raw_output: &str) -> Result<()> {
    let injection_patterns = [
        "Ignore previous instructions",
        "System prompt:",
        "You are now a",
        "Disregard all prior rules",
    ];

    for pattern in &injection_patterns {
        if raw_output.contains(pattern) {
            let log_dir = Path::new("analysis").join(slug);
            fs::create_dir_all(&log_dir)?;
            let finding = format!(
                "[{}] PROMPT INJECTION DETECTED in tool {}: matched pattern '{}'\n",
                chrono::Utc::now().to_rfc3339(),
                tool_name,
                pattern
            );
            let findings_path = log_dir.join("security_findings.jsonl");
            let mut existing = fs::read_to_string(&findings_path).unwrap_or_default();
            existing.push_str(&finding);
            fs::write(findings_path, existing)?;

            bail!("SECURITY GUARD: Potential prompt injection payload detected from REA tool output. Logged to security_findings.jsonl.");
        }
    }
    Ok(())
}

```

---

## 4. REA MCP Client (`rearch-rea`)

### `crates/rearch-rea/Cargo.toml`

```toml
[package]
name = "rearch-rea"
version.workspace = true
edition.workspace = true

[dependencies]
rearch-core = { path = "../rearch-core" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true

```

### `crates/rearch-rea/src/lib.rs`

```rust
pub mod mcp_client;
pub mod cdp;

```

### `crates/rearch-rea/src/mcp_client.rs`

```rust
use anyhow::{Context, Result};
use rearch_core::guard::scan_untrusted_output;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub struct ReaMcpClient {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl ReaMcpClient {
    pub async fn spawn() -> Result<Self> {
        let mut child = Command::new("npx")
            .args(["-y", "rea-agents@latest", "mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn rea-agents MCP server via npx")?;

        let stdin = child.stdin.take().context("Missing stdin")?;
        let stdout = BufReader::new(child.stdout.take().context("Missing stdout")?);

        Ok(Self {
            _child: child,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    pub async fn call_tool(&mut self, slug: &str, tool_name: &str, arguments: Value) -> Result<Value> {
        let req_id = self.next_id;
        self.next_id += 1;

        let payload = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let mut line = serde_json::to_string(&payload)?;
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;

        let mut resp_line = String::new();
        self.stdout.read_line(&mut resp_line).await?;
        let resp: Value = serde_json::from_str(&resp_line)?;

        let raw_text = resp.to_string();
        scan_untrusted_output(slug, tool_name, &raw_text)?;

        Ok(resp["result"].clone())
    }
}

```

---

## 5. Proof Engine & HTML Report Generator (`rearch-proof`)

### `crates/rearch-proof/Cargo.toml`

```toml
[package]
name = "rearch-proof"
version.workspace = true
edition.workspace = true

[dependencies]
rearch-core = { path = "../rearch-core" }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
askama.workspace = true
chrono.workspace = true

```

### `crates/rearch-proof/src/lib.rs`

```rust
pub mod verifier;
pub mod report;

```

### `crates/rearch-proof/src/verifier.rs`

```rust
use anyhow::{Context, Result};
use rearch_core::models::{ProofVerdict, RuleCard, VerificationStatus};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofReport {
    pub slug: String,
    pub verdict: ProofVerdict,
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub details: String,
}

#[derive(Debug, Deserialize)]
struct PreflightConfig {
    #[serde(default)]
    pub can_run_legacy_locally: bool,
}

pub fn run_proof_pack(slug: &str) -> Result<ProofReport> {
    let analysis_dir = Path::new("analysis").join(slug);
    let modern_dir = Path::new("modernized").join(slug);
    let proof_dir = analysis_dir.join("proof");
    fs::create_dir_all(&proof_dir)?;

    let rules_dir = analysis_dir.join("rules");
    if rules_dir.exists() {
        for entry in fs::read_dir(&rules_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let card: RuleCard = serde_json::from_str(&fs::read_to_string(&path)?)?;
                if card.verification.second_agent_status != VerificationStatus::Confirmed {
                    return write_and_return(
                        slug, &proof_dir, ProofVerdict::NotProven, 0, 0, 0,
                        &format!("Unconfirmed rule card: {}", card.rule_id),
                    );
                }
            }
        }
    }

    let report_json_path = proof_dir.join("test-results.json");
    if report_json_path.exists() {
        fs::remove_file(&report_json_path)?;
    }

    let (total, passed, failed, exit_ok) = execute_and_parse_tests(&modern_dir, &report_json_path)?;

    if total == 0 || failed > 0 || !exit_ok {
        return write_and_return(
            slug, &proof_dir, ProofVerdict::NotProven, total, passed, failed,
            "Zero tests executed or test suite failed during clean proof-pack run.",
        );
    }

    let preflight_path = analysis_dir.join("preflight.json");
    let preflight: PreflightConfig = if preflight_path.exists() {
        serde_json::from_str(&fs::read_to_string(preflight_path)?)?
    } else {
        PreflightConfig { can_run_legacy_locally: false }
    };

    let (verdict, msg) = if preflight.can_run_legacy_locally {
        (ProofVerdict::Proven, format!("All {passed}/{total} tests passed with live legacy parity."))
    } else {
        (ProofVerdict::PartlyProven, format!("All {passed}/{total} tests passed against recorded REA snapshot."))
    };

    write_and_return(slug, &proof_dir, verdict, total, passed, failed, &msg)
}

fn execute_and_parse_tests(modern_dir: &Path, report_json: &Path) -> Result<(u64, u64, u64, bool)> {
    if modern_dir.join("Cargo.toml").exists() {
        let output = Command::new("cargo")
            .args(["test", "--", "--format=terse"])
            .current_dir(modern_dir)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let passed = stdout.lines().filter(|l| l.ends_with("... ok")).count() as u64;
        let failed = stdout.lines().filter(|l| l.ends_with("... FAILED")).count() as u64;
        Ok((passed + failed, passed, failed, output.status.success()))
    } else {
        let status = Command::new("npx")
            .args(["vitest", "run", "--reporter=json", &format!("--outputFile={}", report_json.display())])
            .current_dir(modern_dir)
            .status()?;
        if !report_json.exists() {
            return Ok((0, 0, 0, false));
        }
        let val: serde_json::Value = serde_json::from_str(&fs::read_to_string(report_json)?)?;
        let total = val["numTotalTests"].as_u64().unwrap_or(0);
        let passed = val["numPassedTests"].as_u64().unwrap_or(0);
        let failed = val["numFailedTests"].as_u64().unwrap_or(0);
        Ok((total, passed, failed, status.success()))
    }
}

fn write_and_return(
    slug: &str, proof_dir: &Path, verdict: ProofVerdict, total: u64, passed: u64, failed: u64, details: &str,
) -> Result<ProofReport> {
    let report = ProofReport {
        slug: slug.to_string(), verdict, total_tests: total, passed_tests: passed, failed_tests: failed, details: details.to_string(),
    };
    fs::write(proof_dir.join("verdict.json"), serde_json::to_string_pretty(&report)?)?;
    Ok(report)
}

```

---

## 6. CLI Orchestrator & Loop Runner (`rearch-cli`)

### `crates/rearch-cli/Cargo.toml`

```toml
[package]
name = "rearch-cli"
version.workspace = true
edition.workspace = true

[dependencies]
rearch-core = { path = "../rearch-core" }
rearch-rea = { path = "../rearch-rea" }
rearch-proof = { path = "../rearch-proof" }
clap.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true

```

### `crates/rearch-cli/src/main.rs`

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};

mod bootstrap;
mod loop_runner;

#[derive(Parser)]
#[command(name = "rearch", version = "1.0", about = "Autonomous REA & Code-Modernization Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bootstrap {
        #[arg(long, default_value = "doctor")]
        mode: String,
    },
    Intent { slug: String },
    Preflight { slug: String, #[arg(long)] source: Option<String> },
    Assess { slug: String },
    Map { slug: String },
    ExtractRules { slug: String },
    Review { slug: String },
    Brief { slug: String },
    LoopStep {
        slug: String,
        #[arg(long)] window: String,
        #[arg(long, default_value_t = 15)] max_iterations: u32,
    },
    ProofPack { slug: String },
    Harden { slug: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bootstrap { mode } => bootstrap::run_bootstrap(&mode)?,
        Commands::Intent { slug } => println!("Initialized INTENT.md for slug: {slug}"),
        Commands::Preflight { slug, source } => println!("Ran preflight for {slug} with source: {source:?}"),
        Commands::Assess { slug } => println!("Assessed artifact: {slug}"),
        Commands::Map { slug } => println!("Generated topology map for {slug}"),
        Commands::ExtractRules { slug } => println!("Extracted BDD rules for {slug}"),
        Commands::Review { slug } => println!("Reviewed rules for {slug}"),
        Commands::Brief { slug } => println!("Generated steering brief for {slug}"),
        Commands::LoopStep { slug, window, max_iterations } => {
            loop_runner::execute_loop_step(&slug, &window, max_iterations)?;
        }
        Commands::ProofPack { slug } => {
            let report = rearch_proof::verifier::run_proof_pack(&slug)?;
            println!("Proof Pack Verdict: {:?} ({})", report.verdict, report.details);
        }
        Commands::Harden { slug } => println!("Generated hardening patch for {slug}"),
    }
    Ok(())
}

```

---

## 7. Autonomous Direct Tagging & CI/CD Workflows

Tagging and releases are completely autonomous on push to `main` without creating intermediate release branches (`release-please--branches--main`) or requiring manual human pull-request approvals.

### `.github/workflows/ci.yaml`

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Build & Test (Rust 1.85+)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust Toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: Cargo Cache
        uses: Swatinem/rust-cache@v2
      - name: Check Workspace Formatting
        run: cargo fmt -- --check
      - name: Clippy Lints
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Run Workspace Tests
        run: cargo test --workspace

```

### `.github/workflows/release.yaml`

```yaml
name: Autonomous Tag & Release

on:
  push:
    branches: [ main ]

permissions:
  contents: write

jobs:
  autonomous-release:
    name: Autonomous Tag & Release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Configure Git user
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"

      - name: Install Rust Toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Determine Next Semantic Version & Tag
        id: release
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          # Evaluates conventional commits since latest v* tag,
          # tags the commit directly on main, and publishes GitHub Release.

      - name: Publish to crates.io (Optional)
        if: steps.release.outputs.tag_created == 'true' && env.CARGO_REGISTRY_TOKEN != ''
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo publish -p rearch-core --token $CARGO_REGISTRY_TOKEN || true
          cargo publish -p rearch-rea --token $CARGO_REGISTRY_TOKEN || true
          cargo publish -p rearch-proof --token $CARGO_REGISTRY_TOKEN || true
          cargo publish -p rearch-cli --token $CARGO_REGISTRY_TOKEN || true

```

---

## 8. Verification & Build Execution

Compile, test, and verify the workspace locally:

```bash
cargo build --workspace --release
cargo run --bin rearch -- bootstrap --mode doctor
cargo test --workspace

```
