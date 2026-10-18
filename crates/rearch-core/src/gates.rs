use crate::models::{
    Divergence, HardenApproval, PreflightAnswers, ProofVerdictRecord, RuleCard, VerificationStatus,
};
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use walkdir::WalkDir;

/// Gate 1: The 5 Preflight Questions must be signed off by a human.
pub fn verify_gate_1_locked_at(base_dir: &Path, slug: &str) -> Result<()> {
    let preflight_path = base_dir.join("analysis").join(slug).join("preflight.json");
    if !preflight_path.exists() {
        bail!("GATE 1 VIOLATION: preflight.json not found. Run `rearch preflight <slug>` first.");
    }
    let content = fs::read_to_string(&preflight_path)?;
    let answers: PreflightAnswers = serde_json::from_str(&content)
        .context("Failed to parse preflight.json in verify_gate_1")?;

    if !answers.gate_1_approved {
        bail!("GATE 1 VIOLATION: The 5 Preflight Questions have not been signed off by a human.");
    }
    Ok(())
}

pub fn verify_gate_1_locked(slug: &str) -> Result<()> {
    verify_gate_1_locked_at(Path::new("."), slug)
}

/// Gate 2: Disputed Business Rules must be confirmed/resolved by a human.
pub fn verify_gate_2_reviewed_at(base_dir: &Path, slug: &str) -> Result<()> {
    let rules_dir = base_dir.join("analysis").join(slug).join("rules");
    if !rules_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(rules_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let card: RuleCard = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse rule card {}", path.display()))?;
            if card.verification.second_agent_status == VerificationStatus::Disputed
                && card.verification.human_review_status != VerificationStatus::Confirmed
            {
                bail!(
                    "GATE 2 VIOLATION: Disputed rule {} has not been human confirmed.",
                    card.rule_id
                );
            }
        }
    }
    Ok(())
}

pub fn verify_gate_2_reviewed(slug: &str) -> Result<()> {
    verify_gate_2_reviewed_at(Path::new("."), slug)
}

/// Gate 3: Steering Brief sign-off before writing modernized code.
pub fn verify_gate_3_signed_at(base_dir: &Path, slug: &str) -> Result<()> {
    let brief_path = base_dir.join("analysis").join(slug).join("BRIEF.md");
    if !brief_path.exists() {
        bail!("GATE 3 VIOLATION: BRIEF.md not found. Run `rearch brief <slug>` first.");
    }
    let content = fs::read_to_string(&brief_path)?;
    if !content.contains("STATUS: SIGNED_BY_HUMAN") {
        bail!("GATE 3 VIOLATION: Steering Brief BRIEF.md has not been signed off by a human (must contain 'STATUS: SIGNED_BY_HUMAN').");
    }
    Ok(())
}

pub fn verify_gate_3_signed(slug: &str) -> Result<()> {
    verify_gate_3_signed_at(Path::new("."), slug)
}

/// Gate 4: Behavioral divergences between legacy and modern must be explicitly accepted by human.
pub fn verify_gate_4_divergences_at(base_dir: &Path, slug: &str) -> Result<()> {
    let divergences_path = base_dir
        .join("analysis")
        .join(slug)
        .join("accepted_divergences.json");
    if !divergences_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&divergences_path)?;
    let divs: Vec<Divergence> =
        serde_json::from_str(&content).context("Failed to parse accepted_divergences.json")?;

    for div in divs {
        if !div.approved_by_human {
            bail!(
                "GATE 4 VIOLATION: Unapproved divergence '{}' found in accepted_divergences.json.",
                div.divergence_id
            );
        }
    }
    Ok(())
}

pub fn verify_gate_4_divergences(slug: &str) -> Result<()> {
    verify_gate_4_divergences_at(Path::new("."), slug)
}

/// Gate 5: Proof pack sign-off.
pub fn verify_gate_5_proof_signed_at(base_dir: &Path, slug: &str) -> Result<()> {
    let verdict_path = base_dir
        .join("analysis")
        .join(slug)
        .join("proof")
        .join("verdict.json");
    if !verdict_path.exists() {
        bail!("GATE 5 VIOLATION: verdict.json not found in proof directory.");
    }
    let content = fs::read_to_string(&verdict_path)?;
    let record: ProofVerdictRecord =
        serde_json::from_str(&content).context("Failed to parse verdict.json in Gate 5 check")?;

    if !record.signed_by_human {
        bail!("GATE 5 VIOLATION: Proof Pack verdict has not been signed off by a human (signed_by_human: false).");
    }
    Ok(())
}

pub fn verify_gate_5_proof_signed(slug: &str) -> Result<()> {
    verify_gate_5_proof_signed_at(Path::new("."), slug)
}

/// Gate 6: Hardening patch human review & approval.
pub fn verify_gate_6_harden_signed_at(base_dir: &Path, slug: &str) -> Result<()> {
    let harden_path = base_dir
        .join("analysis")
        .join(slug)
        .join("harden_approved.json");
    if !harden_path.exists() {
        bail!("GATE 6 VIOLATION: harden_approved.json not found. Hardening patch must be human reviewed and signed.");
    }
    let content = fs::read_to_string(&harden_path)?;
    let approval: HardenApproval =
        serde_json::from_str(&content).context("Failed to parse harden_approved.json")?;

    if !approval.approved_by_human {
        bail!("GATE 6 VIOLATION: Hardening patch not approved by human.");
    }
    Ok(())
}

pub fn verify_gate_6_harden_signed(slug: &str) -> Result<()> {
    verify_gate_6_harden_signed_at(Path::new("."), slug)
}

/// Computes a deterministic SHA-256 hash for a file or a directory tree.
pub fn compute_legacy_digest(path: &Path) -> Result<String> {
    if !path.exists() {
        bail!("Path {} does not exist", path.display());
    }

    let mut hasher = Sha256::new();

    if path.is_file() {
        let bytes = fs::read(path)?;
        hasher.update(&bytes);
    } else if path.is_dir() {
        let mut entries: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        // Sort by relative path to ensure deterministic hashing order
        entries.sort_by_key(|e| e.path().to_path_buf());

        for entry in entries {
            let rel = entry.path().strip_prefix(path).unwrap_or(entry.path());
            hasher.update(rel.to_string_lossy().as_bytes());
            let bytes = fs::read(entry.path())?;
            hasher.update(&bytes);
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies that legacy/<slug> has not been modified since preflight lock.
pub fn verify_legacy_unmodified_at(base_dir: &Path, slug: &str) -> Result<()> {
    let preflight_path = base_dir.join("analysis").join(slug).join("preflight.json");
    if !preflight_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&preflight_path)?;
    let answers: PreflightAnswers = serde_json::from_str(&content)?;

    if let Some(expected_digest) = answers.legacy_sha256 {
        let legacy_path = base_dir.join("legacy").join(slug);
        if !legacy_path.exists() {
            bail!(
                "LEGACY IMMUTABILITY VIOLATION: legacy/{} was removed!",
                slug
            );
        }
        let current_digest = compute_legacy_digest(&legacy_path)?;
        if current_digest != expected_digest {
            bail!(
                "LEGACY IMMUTABILITY VIOLATION: MUTATION DETECTED in legacy/{}. Expected digest {}, got {}",
                slug,
                expected_digest,
                current_digest
            );
        }
    }
    Ok(())
}

pub fn verify_legacy_unmodified(slug: &str) -> Result<()> {
    verify_legacy_unmodified_at(Path::new("."), slug)
}

/// Enforces REA Law 2 (Smallest Useful Inventory First):
/// Blocks deep function decompilation or full-bundle AST dumps until analysis/<slug>/map.json is populated.
pub fn verify_inventory_populated_at(base_dir: &Path, slug: &str) -> Result<()> {
    let map_path = base_dir.join("analysis").join(slug).join("map.json");
    if !map_path.exists() {
        bail!(
            "REA LAW 2 VIOLATION (Smallest Inventory First): analysis/{slug}/map.json not found. Populate top-level inventory before deep inspection."
        );
    }
    let content = fs::read_to_string(&map_path)?;
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        bail!(
            "REA LAW 2 VIOLATION (Smallest Inventory First): analysis/{slug}/map.json is empty. Populate top-level inventory before deep inspection."
        );
    }
    Ok(())
}

pub fn verify_inventory_populated(slug: &str) -> Result<()> {
    verify_inventory_populated_at(Path::new("."), slug)
}
