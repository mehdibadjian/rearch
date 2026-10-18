use rearch_core::models::*;
use rearch_proof::report::render_html_report_at;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_render_html_report() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    let rules_dir = analysis_dir.join("rules");
    let proof_dir = analysis_dir.join("proof");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::create_dir_all(&proof_dir).unwrap();

    // Map file
    fs::write(
        analysis_dir.join("map.json"),
        r#"{"nodes": ["auth", "billing"], "edges": [{"from": "auth", "to": "billing"}]}"#,
    )
    .unwrap();

    // Verdict file
    fs::write(
        proof_dir.join("verdict.json"),
        r#"{
            "slug": "sample",
            "verdict": "PROVEN",
            "total_tests": 12,
            "passed_tests": 12,
            "failed_tests": 0,
            "details": "100% parity verified"
        }"#,
    )
    .unwrap();

    // A rule card
    let card = RuleCard {
        rule_id: "RULE-01".to_string(),
        module: "auth".to_string(),
        title: "Token validation".to_string(),
        bdd: BddSpec {
            given: "valid JWT".to_string(),
            when: "calling endpoint".to_string(),
            then: "permit request".to_string(),
        },
        provenance: Provenance {
            epistemic_tier: EpistemicTier::Observation,
            source_citations: vec!["legacy/jwt.rs:40".to_string()],
            comment_discrepancy_flagged: false,
        },
        verification: RuleVerification {
            extractor_agent: "rule-extractor".to_string(),
            verifier_agent: "rule-verifier".to_string(),
            second_agent_status: VerificationStatus::Confirmed,
            human_review_status: VerificationStatus::Confirmed,
        },
    };
    fs::write(rules_dir.join("rule-01.json"), serde_json::to_string(&card).unwrap()).unwrap();

    let html = render_html_report_at(temp.path(), "sample").unwrap();
    assert!(html.contains("Modernization Proof Report: sample"));
    assert!(html.contains("PROVEN"));
    assert!(html.contains("RULE-01"));
    assert!(html.contains("Token validation"));

    // Check REPORT.html was saved
    let report_path = analysis_dir.join("REPORT.html");
    assert!(report_path.exists());
}
