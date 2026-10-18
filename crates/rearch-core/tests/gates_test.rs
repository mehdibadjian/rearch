use rearch_core::gates::*;
use rearch_core::models::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_gate_1_fails_when_preflight_missing() {
    let temp = tempdir().unwrap();
    let res = verify_gate_1_locked_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("preflight.json not found"));
}

#[test]
fn test_gate_1_fails_when_not_approved() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": false, "can_run_legacy_locally": true}"#,
    )
    .unwrap();

    let res = verify_gate_1_locked_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("not been signed off"));
}

#[test]
fn test_gate_1_passes_when_approved() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&analysis_dir).unwrap();
    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true, "can_run_legacy_locally": true}"#,
    )
    .unwrap();

    assert!(verify_gate_1_locked_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_gate_2_disputed_rules_must_be_human_reviewed() {
    let temp = tempdir().unwrap();
    let rules_dir = temp.path().join("analysis/my-service/rules");
    fs::create_dir_all(&rules_dir).unwrap();

    let unreviewed_card = RuleCard {
        rule_id: "RULE-01".to_string(),
        module: "billing".to_string(),
        title: "Tax rate".to_string(),
        bdd: BddSpec {
            given: "user in CA".to_string(),
            when: "checkout".to_string(),
            then: "apply 7.25%".to_string(),
        },
        provenance: Provenance {
            epistemic_tier: EpistemicTier::Observation,
            source_citations: vec!["legacy/tax.js".to_string()],
            comment_discrepancy_flagged: false,
        },
        verification: RuleVerification {
            extractor_agent: "rule-extractor".to_string(),
            verifier_agent: "rule-verifier".to_string(),
            second_agent_status: VerificationStatus::Disputed,
            human_review_status: VerificationStatus::Pending,
        },
    };

    fs::write(
        rules_dir.join("rule-01.json"),
        serde_json::to_string(&unreviewed_card).unwrap(),
    )
    .unwrap();

    let res = verify_gate_2_reviewed_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Disputed rule RULE-01 has not been human confirmed"));

    // Now mark human confirmed
    let mut reviewed_card = unreviewed_card;
    reviewed_card.verification.human_review_status = VerificationStatus::Confirmed;
    fs::write(
        rules_dir.join("rule-01.json"),
        serde_json::to_string(&reviewed_card).unwrap(),
    )
    .unwrap();

    assert!(verify_gate_2_reviewed_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_gate_3_requires_signed_brief() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&analysis_dir).unwrap();

    // Missing brief
    assert!(verify_gate_3_signed_at(temp.path(), "my-service").is_err());

    // Unsigned brief
    fs::write(
        analysis_dir.join("BRIEF.md"),
        "# Modernization Steering Brief\nSTATUS: DRAFT\n",
    )
    .unwrap();
    let res = verify_gate_3_signed_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("STATUS: SIGNED_BY_HUMAN"));

    // Signed brief
    fs::write(
        analysis_dir.join("BRIEF.md"),
        "# Modernization Steering Brief\nSTATUS: SIGNED_BY_HUMAN\nSigned-By: Engineer",
    )
    .unwrap();
    assert!(verify_gate_3_signed_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_gate_4_divergences_check() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&analysis_dir).unwrap();

    // No divergences file: passes (no divergences detected)
    assert!(verify_gate_4_divergences_at(temp.path(), "my-service").is_ok());

    // Unapproved divergence
    fs::write(
        analysis_dir.join("accepted_divergences.json"),
        r#"[{"divergence_id": "DIV-01", "approved_by_human": false}]"#,
    )
    .unwrap();
    let res = verify_gate_4_divergences_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Unapproved divergence"));

    // Approved divergence
    fs::write(
        analysis_dir.join("accepted_divergences.json"),
        r#"[{"divergence_id": "DIV-01", "approved_by_human": true}]"#,
    )
    .unwrap();
    assert!(verify_gate_4_divergences_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_gate_5_proof_signoff() {
    let temp = tempdir().unwrap();
    let proof_dir = temp.path().join("analysis/my-service/proof");
    fs::create_dir_all(&proof_dir).unwrap();

    // Unsigned
    fs::write(
        proof_dir.join("verdict.json"),
        r#"{"verdict": "PROVEN", "signed_by_human": false}"#,
    )
    .unwrap();
    assert!(verify_gate_5_proof_signed_at(temp.path(), "my-service").is_err());

    // Signed
    fs::write(
        proof_dir.join("verdict.json"),
        r#"{"verdict": "PROVEN", "signed_by_human": true}"#,
    )
    .unwrap();
    assert!(verify_gate_5_proof_signed_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_gate_6_harden_signed() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&analysis_dir).unwrap();

    // Missing harden approval
    assert!(verify_gate_6_harden_signed_at(temp.path(), "my-service").is_err());

    // Signed harden approval
    fs::write(
        analysis_dir.join("harden_approved.json"),
        r#"{"approved_by_human": true}"#,
    )
    .unwrap();
    assert!(verify_gate_6_harden_signed_at(temp.path(), "my-service").is_ok());
}

#[test]
fn test_legacy_immutability_verification() {
    let temp = tempdir().unwrap();
    let legacy_dir = temp.path().join("legacy/my-service");
    let analysis_dir = temp.path().join("analysis/my-service");
    fs::create_dir_all(&legacy_dir).unwrap();
    fs::create_dir_all(&analysis_dir).unwrap();

    let file_path = legacy_dir.join("original.js");
    fs::write(&file_path, "console.log('original');").unwrap();

    let digest = compute_legacy_digest(&legacy_dir).unwrap();

    // Record digest in preflight.json
    fs::write(
        analysis_dir.join("preflight.json"),
        serde_json::json!({
            "gate_1_approved": true,
            "legacy_sha256": digest
        })
        .to_string(),
    )
    .unwrap();

    // Initial check: matches
    assert!(verify_legacy_unmodified_at(temp.path(), "my-service").is_ok());

    // Mutate legacy file!
    fs::write(&file_path, "console.log('mutated!');").unwrap();

    // Verification must fail!
    let res = verify_legacy_unmodified_at(temp.path(), "my-service");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("MUTATION DETECTED"));
}
