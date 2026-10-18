use anyhow::{Context, Result};
use rearch_core::gates::{compute_legacy_digest, verify_gate_2_reviewed, verify_gate_3_signed};
use rearch_core::models::PreflightAnswers;
use rearch_proof::report::render_html_report;
use std::{fs, path::Path};

pub fn run_intent(slug: &str) -> Result<()> {
    let intent_file = Path::new("INTENT.md");
    if !intent_file.exists() {
        let content = format!(
            "# Modernization Intent\n\nTarget Slug: `{slug}`\nTimestamp: {}\nStatus: INITIALIZED\n",
            chrono::Utc::now().to_rfc3339()
        );
        fs::write(intent_file, content)?;
        println!("Created root INTENT.md for target slug: {slug}");
    } else {
        println!("INTENT.md already exists for target slug: {slug}");
    }
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_preflight(slug: &str, source: Option<&str>) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;

    let legacy_dir = Path::new("legacy").join(slug);
    if let Some(src) = source {
        if !legacy_dir.exists() {
            fs::create_dir_all(&legacy_dir)?;
            let src_path = Path::new(src);
            if src_path.is_file() {
                let file_name = src_path.file_name().context("Invalid source file name")?;
                fs::copy(src_path, legacy_dir.join(file_name))?;
            }
        }
    }

    let legacy_digest = if legacy_dir.exists() {
        compute_legacy_digest(&legacy_dir).ok()
    } else {
        None
    };

    let preflight = PreflightAnswers {
        whole_or_slice: Some("slice".to_string()),
        can_run_legacy_locally: true,
        custom_build_tooling: None,
        prior_attempts: None,
        off_limits_components: vec![],
        gate_1_approved: false,
        legacy_sha256: legacy_digest,
    };

    let preflight_path = analysis_dir.join("preflight.json");
    if !preflight_path.exists() {
        fs::write(&preflight_path, serde_json::to_string_pretty(&preflight)?)?;
        println!(
            "Generated {} (Gate 1 sign-off pending)",
            preflight_path.display()
        );
    } else {
        println!("Preflight already exists at {}", preflight_path.display());
    }
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_assess(slug: &str) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let evidence_path = analysis_dir.join("EVIDENCE_STATE.md");
    if !evidence_path.exists() {
        let template = format!(
            "# REA Evidence State: {slug}\n\n| Item | Tier | Description | Citations |\n| --- | --- | --- | --- |\n| **Q-01** | `UNKNOWN` | System entrypoint and route topology | TBD |\n"
        );
        fs::write(&evidence_path, template)?;
        println!("Initialized {}", evidence_path.display());
    }
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_map(slug: &str) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let map_path = analysis_dir.join("map.json");
    if !map_path.exists() {
        let default_map = serde_json::json!({
            "slug": slug,
            "modules": [],
            "dependencies": [],
            "endpoints": []
        });
        fs::write(&map_path, serde_json::to_string_pretty(&default_map)?)?;
        println!("Generated {}", map_path.display());
    }
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_extract_rules(slug: &str) -> Result<()> {
    let rules_dir = Path::new("analysis").join(slug).join("rules");
    fs::create_dir_all(&rules_dir)?;
    println!("Rules directory initialized at {}", rules_dir.display());
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_review(slug: &str) -> Result<()> {
    verify_gate_2_reviewed(slug)?;
    println!("Gate 2 Review complete: All disputed rule cards have been resolved for {slug}");
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_brief(slug: &str) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let brief_path = analysis_dir.join("BRIEF.md");
    if !brief_path.exists() {
        let template = format!(
            "# Modernization Steering Brief: {slug}\n\nSTATUS: DRAFT\n\n## 1. Executive Summary\n\n## 2. Phased Modernization Plan\n\n## 3. Architecture Critic Review\n\n---\n*Sign off with 'STATUS: SIGNED_BY_HUMAN' to unlock Gate 3.*\n"
        );
        fs::write(&brief_path, template)?;
        println!("Created draft steering brief at {}", brief_path.display());
    }
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_sign(slug: &str) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let brief_path = analysis_dir.join("BRIEF.md");
    if brief_path.exists() {
        let content = fs::read_to_string(&brief_path)?;
        let updated = content.replace("STATUS: DRAFT", "STATUS: SIGNED_BY_HUMAN");
        let final_content = if updated.contains("STATUS: SIGNED_BY_HUMAN") {
            updated
        } else {
            format!("{content}\nSTATUS: SIGNED_BY_HUMAN\n")
        };
        fs::write(&brief_path, final_content)?;
    } else {
        fs::write(
            &brief_path,
            format!("# Modernization Steering Brief: {slug}\n\nSTATUS: SIGNED_BY_HUMAN\n"),
        )?;
    }
    println!(
        "Signed Steering Brief at {} (Gate 3 unlocked)",
        brief_path.display()
    );
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_build_stage(slug: &str, mode: &str) -> Result<()> {
    verify_gate_3_signed(slug)?;
    let modern_dir = Path::new("modernized").join(slug);
    fs::create_dir_all(&modern_dir)?;
    println!(
        "Initialized Stage 6 ({mode}) workspace at {}",
        modern_dir.display()
    );
    let _ = render_html_report(slug)?;
    Ok(())
}

pub fn run_harden(slug: &str) -> Result<()> {
    let analysis_dir = Path::new("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;
    let patch_path = analysis_dir.join("harden.patch");
    if !patch_path.exists() {
        fs::write(&patch_path, "# Hardening patch placeholder\n")?;
    }
    let approved_path = analysis_dir.join("harden_approved.json");
    if !approved_path.exists() {
        let content = serde_json::json!({
            "approved_by_human": false,
            "patch_file": "harden.patch",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        fs::write(&approved_path, serde_json::to_string_pretty(&content)?)?;
        println!("Created hardening artifacts in {}", analysis_dir.display());
    }
    let _ = render_html_report(slug)?;
    Ok(())
}
