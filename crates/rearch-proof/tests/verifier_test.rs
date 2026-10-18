use rearch_core::gates::compute_legacy_digest;
use rearch_core::models::*;
use rearch_proof::verifier::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_verifier_fails_on_unconfirmed_rule_cards() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let rules_dir = analysis_dir.join("rules");
    let proof_dir = analysis_dir.join("proof");
    let modern_dir = temp.path().join("modernized/sample");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::create_dir_all(&proof_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();

    let card = RuleCard {
        rule_id: "RULE-UNCONFIRMED".to_string(),
        module: "billing".to_string(),
        title: "Disputed rule".to_string(),
        bdd: BddSpec {
            given: "g".to_string(),
            when: "w".to_string(),
            then: "t".to_string(),
        },
        provenance: Provenance {
            epistemic_tier: EpistemicTier::Observation,
            source_citations: vec![],
            comment_discrepancy_flagged: false,
        },
        verification: RuleVerification {
            extractor_agent: "extractor".to_string(),
            verifier_agent: "verifier".to_string(),
            second_agent_status: VerificationStatus::Disputed,
            human_review_status: VerificationStatus::Pending,
        },
    };

    fs::write(
        rules_dir.join("rule.json"),
        serde_json::to_string(&card).unwrap(),
    )
    .unwrap();

    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::NotProven);
    assert!(report.details.contains("Unconfirmed rule card"));

    let verdict_file = proof_dir.join("verdict.json");
    assert!(verdict_file.exists());
}

#[test]
fn test_verifier_fails_on_zero_tests() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let modern_dir = temp.path().join("modernized/sample");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();

    // No tests exist in modernized/sample
    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::NotProven);
    assert_eq!(report.total_tests, 0);
    assert!(report.details.contains("Zero tests executed"));
}

#[test]
fn test_verifier_produces_proven_when_legacy_runs_locally() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let modern_dir = temp.path().join("modernized/sample");
    let proof_dir = analysis_dir.join("proof");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();
    fs::create_dir_all(&proof_dir).unwrap();

    // Preflight indicates can_run_legacy_locally: true
    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true, "can_run_legacy_locally": true}"#,
    )
    .unwrap();

    // Mock test runner output via test-results.json in modernized dir
    fs::write(
        modern_dir.join("test-results.json"),
        r#"{"numTotalTests": 10, "numPassedTests": 10, "numFailedTests": 0}"#,
    )
    .unwrap();

    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::Proven);
    assert_eq!(report.total_tests, 10);
    assert_eq!(report.passed_tests, 10);
    assert_eq!(report.failed_tests, 0);
    assert!(report.details.contains("live legacy parity"));
}

#[test]
fn test_verifier_produces_partly_proven_when_legacy_cannot_run_locally() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let modern_dir = temp.path().join("modernized/sample");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();

    // Preflight indicates can_run_legacy_locally: false
    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true, "can_run_legacy_locally": false}"#,
    )
    .unwrap();

    fs::write(
        modern_dir.join("test-results.json"),
        r#"{"numTotalTests": 5, "numPassedTests": 5, "numFailedTests": 0}"#,
    )
    .unwrap();

    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::PartlyProven);
    assert_eq!(report.total_tests, 5);
    assert!(report.details.contains("recorded REA snapshot"));
}

#[test]
fn test_verifier_parses_junit_xml_results() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let modern_dir = temp.path().join("modernized/sample");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();

    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true, "can_run_legacy_locally": true}"#,
    )
    .unwrap();

    let junit_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
    <testsuite name="auth_suite" tests="4" failures="0" errors="0">
        <testcase name="test_login"/>
    </testsuite>
    <testsuite name="billing_suite" tests="3" failures="0" errors="0">
        <testcase name="test_invoice"/>
    </testsuite>
</testsuites>
"#;
    fs::write(modern_dir.join("junit.xml"), junit_xml).unwrap();

    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::Proven);
    assert_eq!(report.total_tests, 7);
    assert_eq!(report.passed_tests, 7);
    assert_eq!(report.failed_tests, 0);
}

#[test]
fn test_verifier_fails_when_legacy_mutated() {
    let temp = tempdir().unwrap();
    let legacy_dir = temp.path().join("legacy/sample");
    let analysis_dir = temp.path().join("analysis/sample");
    let modern_dir = temp.path().join("modernized/sample");
    fs::create_dir_all(&legacy_dir).unwrap();
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::create_dir_all(&modern_dir).unwrap();

    fs::write(legacy_dir.join("app.c"), "int main() { return 0; }").unwrap();
    let digest = compute_legacy_digest(&legacy_dir).unwrap();

    fs::write(
        analysis_dir.join("preflight.json"),
        serde_json::json!({
            "gate_1_approved": true,
            "can_run_legacy_locally": true,
            "legacy_sha256": digest
        })
        .to_string(),
    )
    .unwrap();

    // Mutate legacy!
    fs::write(legacy_dir.join("app.c"), "int main() { return 1; }").unwrap();

    let report = run_proof_pack_at(temp.path(), "sample").unwrap();
    assert_eq!(report.verdict, ProofVerdict::NotProven);
    assert!(report.details.contains("Legacy immutability violation"));
}
