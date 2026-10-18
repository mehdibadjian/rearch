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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BddSpec {
    pub given: String,
    pub when: String,
    pub then: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub epistemic_tier: EpistemicTier,
    pub source_citations: Vec<String>,
    pub comment_discrepancy_flagged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleVerification {
    pub extractor_agent: String,
    pub verifier_agent: String,
    pub second_agent_status: VerificationStatus,
    pub human_review_status: VerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightAnswers {
    #[serde(default)]
    pub whole_or_slice: Option<String>,
    #[serde(default)]
    pub can_run_legacy_locally: bool,
    #[serde(default)]
    pub custom_build_tooling: Option<String>,
    #[serde(default)]
    pub prior_attempts: Option<String>,
    #[serde(default)]
    pub off_limits_components: Vec<String>,
    #[serde(default)]
    pub gate_1_approved: bool,
    #[serde(default)]
    pub legacy_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub divergence_id: String,
    pub description: Option<String>,
    #[serde(default)]
    pub approved_by_human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerdictRecord {
    pub verdict: ProofVerdict,
    #[serde(default)]
    pub signed_by_human: bool,
    #[serde(default)]
    pub total_tests: u64,
    #[serde(default)]
    pub passed_tests: u64,
    #[serde(default)]
    pub failed_tests: u64,
    #[serde(default)]
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardenApproval {
    #[serde(default)]
    pub approved_by_human: bool,
    #[serde(default)]
    pub patch_file: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}
