use anyhow::{Context, Result};
use askama::Template;
use rearch_core::models::RuleCard;
use serde_json::Value;
use std::{fs, path::Path};

pub struct RuleView {
    pub rule_id: String,
    pub module: String,
    pub tier: String,
    pub title: String,
    pub given: String,
    pub when: String,
    pub then: String,
    pub citations: String,
    pub verifier_agent: String,
    pub second_agent_status: String,
}

#[derive(Template)]
#[template(path = "report.html")]
pub struct ReportTemplate {
    pub slug: String,
    pub timestamp: String,
    pub verdict_str: String,
    pub badge_color: String,
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub details: String,
    pub rules: Vec<RuleView>,
    pub map_summary: String,
}

pub fn render_html_report(slug: &str) -> Result<String> {
    render_html_report_at(Path::new("."), slug)
}

pub fn render_html_report_at(base_dir: &Path, slug: &str) -> Result<String> {
    let analysis_dir = base_dir.join("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;

    let verdict_path = analysis_dir.join("proof").join("verdict.json");
    let (verdict_str, total_tests, passed_tests, failed_tests, details) = if verdict_path.exists() {
        let content = fs::read_to_string(&verdict_path).unwrap_or_default();
        let val: Value = serde_json::from_str(&content).unwrap_or_default();
        (
            val["verdict"].as_str().unwrap_or("PENDING").to_string(),
            val["total_tests"].as_u64().unwrap_or(0),
            val["passed_tests"].as_u64().unwrap_or(0),
            val["failed_tests"].as_u64().unwrap_or(0),
            val["details"]
                .as_str()
                .unwrap_or("No details available")
                .to_string(),
        )
    } else {
        (
            "PENDING".to_string(),
            0,
            0,
            0,
            "Proof pack not yet executed".to_string(),
        )
    };

    let mut rules_raw = Vec::new();
    let rules_dir = analysis_dir.join("rules");
    if rules_dir.exists() {
        if let Ok(entries) = fs::read_dir(rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(card) = serde_json::from_str::<RuleCard>(&content) {
                            rules_raw.push(card);
                        }
                    }
                }
            }
        }
    }
    rules_raw.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));

    let rules: Vec<RuleView> = rules_raw
        .into_iter()
        .map(|r| RuleView {
            rule_id: r.rule_id,
            module: r.module,
            tier: format!("{:?}", r.provenance.epistemic_tier),
            title: r.title,
            given: r.bdd.given,
            when: r.bdd.when,
            then: r.bdd.then,
            citations: r.provenance.source_citations.join(", "),
            verifier_agent: r.verification.verifier_agent,
            second_agent_status: format!("{:?}", r.verification.second_agent_status),
        })
        .collect();

    let map_path = analysis_dir.join("map.json");
    let map_summary = if map_path.exists() {
        fs::read_to_string(map_path).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    };

    let badge_color = match verdict_str.as_str() {
        "PROVEN" => "#10b981",
        "PARTLY PROVEN" => "#3b82f6",
        "NOT PROVEN" => "#ef4444",
        _ => "#6b7280",
    }
    .to_string();

    let tmpl = ReportTemplate {
        slug: slug.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        verdict_str,
        badge_color,
        total_tests,
        passed_tests,
        failed_tests,
        details,
        rules,
        map_summary,
    };

    let html = tmpl
        .render()
        .context("Failed to render Askama HTML report")?;

    let report_path = analysis_dir.join("REPORT.html");
    fs::write(&report_path, &html)
        .with_context(|| format!("Failed to write HTML report to {}", report_path.display()))?;

    Ok(html)
}
