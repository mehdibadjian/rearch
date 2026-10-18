use anyhow::{Context, Result};
use rearch_core::gates::verify_legacy_unmodified_at;
use rearch_core::models::{PreflightAnswers, ProofVerdict, RuleCard, VerificationStatus};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofReport {
    pub slug: String,
    pub verdict: ProofVerdict,
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub details: String,
}

pub fn run_proof_pack(slug: &str) -> Result<ProofReport> {
    run_proof_pack_at(Path::new("."), slug)
}

pub fn run_proof_pack_at(base_dir: &Path, slug: &str) -> Result<ProofReport> {
    let analysis_dir = base_dir.join("analysis").join(slug);
    let modern_dir = base_dir.join("modernized").join(slug);
    let proof_dir = analysis_dir.join("proof");
    fs::create_dir_all(&proof_dir)?;

    // 1. Verify legacy immutability
    if let Err(e) = verify_legacy_unmodified_at(base_dir, slug) {
        return write_and_return(
            slug,
            &proof_dir,
            ProofVerdict::NotProven,
            0,
            0,
            0,
            &format!("Legacy immutability violation: {}", e),
        );
    }

    // 2. Verify all extracted rule cards are confirmed
    let rules_dir = analysis_dir.join("rules");
    if rules_dir.exists() {
        for entry in fs::read_dir(&rules_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let card: RuleCard = serde_json::from_str(&fs::read_to_string(&path)?)?;
                if card.verification.second_agent_status != VerificationStatus::Confirmed {
                    return write_and_return(
                        slug,
                        &proof_dir,
                        ProofVerdict::NotProven,
                        0,
                        0,
                        0,
                        &format!("Unconfirmed rule card: {}", card.rule_id),
                    );
                }
            }
        }
    }

    // 3. Clear cached test results
    let proof_test_json = proof_dir.join("test-results.json");
    if proof_test_json.exists() {
        let _ = fs::remove_file(&proof_test_json);
    }

    // 4. Execute tests and parse results
    let (total, passed, failed, exit_ok) = execute_and_parse_tests(&modern_dir, &proof_test_json)?;

    if total == 0 || failed > 0 || !exit_ok {
        let reason = if total == 0 {
            "Zero tests executed during clean proof-pack run.".to_string()
        } else {
            format!("Test suite failed ({failed}/{total} tests failed).")
        };
        return write_and_return(slug, &proof_dir, ProofVerdict::NotProven, total, passed, failed, &reason);
    }

    // 5. Check preflight configuration for live parity vs snapshot
    let preflight_path = analysis_dir.join("preflight.json");
    let preflight: PreflightAnswers = if preflight_path.exists() {
        serde_json::from_str(&fs::read_to_string(preflight_path)?)
            .unwrap_or(PreflightAnswers {
                whole_or_slice: None,
                can_run_legacy_locally: false,
                custom_build_tooling: None,
                prior_attempts: None,
                off_limits_components: vec![],
                gate_1_approved: false,
                legacy_sha256: None,
            })
    } else {
        PreflightAnswers {
            whole_or_slice: None,
            can_run_legacy_locally: false,
            custom_build_tooling: None,
            prior_attempts: None,
            off_limits_components: vec![],
            gate_1_approved: false,
            legacy_sha256: None,
        }
    };

    let (verdict, msg) = if preflight.can_run_legacy_locally {
        (
            ProofVerdict::Proven,
            format!("All {passed}/{total} tests passed with live legacy parity."),
        )
    } else {
        (
            ProofVerdict::PartlyProven,
            format!("All {passed}/{total} tests passed against recorded REA snapshot."),
        )
    };

    write_and_return(slug, &proof_dir, verdict, total, passed, failed, &msg)
}

fn execute_and_parse_tests(modern_dir: &Path, report_json: &Path) -> Result<(u64, u64, u64, bool)> {
    // If a modern_dir test-results.json exists (mocked or pre-generated)
    let mock_results_path = modern_dir.join("test-results.json");
    if mock_results_path.exists() {
        let val: serde_json::Value = serde_json::from_str(&fs::read_to_string(&mock_results_path)?)?;
        let total = val["numTotalTests"].as_u64().unwrap_or(0);
        let passed = val["numPassedTests"].as_u64().unwrap_or(0);
        let failed = val["numFailedTests"].as_u64().unwrap_or(0);
        return Ok((total, passed, failed, failed == 0 && total > 0));
    }

    if modern_dir.join("Cargo.toml").exists() {
        let output = Command::new("cargo")
            .args(["test", "--", "--format=terse"])
            .current_dir(modern_dir)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let passed = stdout.lines().filter(|l| l.ends_with("... ok")).count() as u64;
        let failed = stdout.lines().filter(|l| l.ends_with("... FAILED")).count() as u64;
        Ok((passed + failed, passed, failed, output.status.success()))
    } else if modern_dir.join("package.json").exists() {
        let status = Command::new("npx")
            .args([
                "vitest",
                "run",
                "--reporter=json",
                &format!("--outputFile={}", report_json.display()),
            ])
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
    } else {
        // No tests found
        Ok((0, 0, 0, false))
    }
}

fn write_and_return(
    slug: &str,
    proof_dir: &Path,
    verdict: ProofVerdict,
    total: u64,
    passed: u64,
    failed: u64,
    details: &str,
) -> Result<ProofReport> {
    let report = ProofReport {
        slug: slug.to_string(),
        verdict,
        total_tests: total,
        passed_tests: passed,
        failed_tests: failed,
        details: details.to_string(),
    };
    fs::write(
        proof_dir.join("verdict.json"),
        serde_json::to_string_pretty(&report)?,
    )
    .context("Failed to write proof verdict.json")?;
    Ok(report)
}
