use rearch_core::models::*;

#[test]
fn test_epistemic_tier_serialization() {
    assert_eq!(
        serde_json::to_string(&EpistemicTier::Observation).unwrap(),
        "\"OBSERVATION\""
    );
    assert_eq!(
        serde_json::to_string(&EpistemicTier::Derivation).unwrap(),
        "\"DERIVATION\""
    );
    assert_eq!(
        serde_json::to_string(&EpistemicTier::Inference).unwrap(),
        "\"INFERENCE\""
    );
    assert_eq!(
        serde_json::to_string(&EpistemicTier::Unknown).unwrap(),
        "\"UNKNOWN\""
    );

    let parsed: EpistemicTier = serde_json::from_str("\"OBSERVATION\"").unwrap();
    assert_eq!(parsed, EpistemicTier::Observation);
}

#[test]
fn test_verification_status_serialization() {
    assert_eq!(
        serde_json::to_string(&VerificationStatus::Confirmed).unwrap(),
        "\"CONFIRMED\""
    );
    assert_eq!(
        serde_json::to_string(&VerificationStatus::Disputed).unwrap(),
        "\"DISPUTED\""
    );
    assert_eq!(
        serde_json::to_string(&VerificationStatus::Pending).unwrap(),
        "\"PENDING\""
    );
}

#[test]
fn test_proof_verdict_serialization() {
    assert_eq!(
        serde_json::to_string(&ProofVerdict::Proven).unwrap(),
        "\"PROVEN\""
    );
    assert_eq!(
        serde_json::to_string(&ProofVerdict::PartlyProven).unwrap(),
        "\"PARTLY PROVEN\""
    );
    assert_eq!(
        serde_json::to_string(&ProofVerdict::NotProven).unwrap(),
        "\"NOT PROVEN\""
    );

    let parsed: ProofVerdict = serde_json::from_str("\"PARTLY PROVEN\"").unwrap();
    assert_eq!(parsed, ProofVerdict::PartlyProven);
}

#[test]
fn test_rule_card_roundtrip() {
    let card = RuleCard {
        rule_id: "RULE-AUTH-001".to_string(),
        module: "auth".to_string(),
        title: "Session token expiry".to_string(),
        bdd: BddSpec {
            given: "an expired session token".to_string(),
            when: "a request is received".to_string(),
            then: "return HTTP 401 Unauthorized".to_string(),
        },
        provenance: Provenance {
            epistemic_tier: EpistemicTier::Observation,
            source_citations: vec!["legacy/auth.c:142".to_string()],
            comment_discrepancy_flagged: false,
        },
        verification: RuleVerification {
            extractor_agent: "rule-extractor".to_string(),
            verifier_agent: "rule-verifier".to_string(),
            second_agent_status: VerificationStatus::Confirmed,
            human_review_status: VerificationStatus::Confirmed,
        },
    };

    let serialized = serde_json::to_string_pretty(&card).unwrap();
    let deserialized: RuleCard = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.rule_id, "RULE-AUTH-001");
    assert_eq!(deserialized.bdd.then, "return HTTP 401 Unauthorized");
    assert_eq!(deserialized.provenance.epistemic_tier, EpistemicTier::Observation);
    assert_eq!(deserialized.verification.second_agent_status, VerificationStatus::Confirmed);
}
