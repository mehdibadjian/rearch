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

    fs::write(rules_dir.join("rule.json"), serde_json::to_string(&card).unwrap()).unwrap();

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

    // Mock test runner output via test-results.json in modernized dir or synthetic cargo test
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
