use anyhow::{Context, Result};
use rearch_core::models::RuleCard;
use serde_json::Value;
use std::{fs, path::Path};

pub fn render_html_report(slug: &str) -> Result<String> {
    render_html_report_at(Path::new("."), slug)
}

pub fn render_html_report_at(base_dir: &Path, slug: &str) -> Result<String> {
    let analysis_dir = base_dir.join("analysis").join(slug);
    fs::create_dir_all(&analysis_dir)?;

    // Read verdict if present
    let verdict_path = analysis_dir.join("proof").join("verdict.json");
    let (verdict_str, total_tests, passed_tests, failed_tests, details) = if verdict_path.exists() {
        let content = fs::read_to_string(&verdict_path).unwrap_or_default();
        let val: Value = serde_json::from_str(&content).unwrap_or_default();
        (
            val["verdict"].as_str().unwrap_or("PENDING").to_string(),
            val["total_tests"].as_u64().unwrap_or(0),
            val["passed_tests"].as_u64().unwrap_or(0),
            val["failed_tests"].as_u64().unwrap_or(0),
            val["details"].as_str().unwrap_or("No details available").to_string(),
        )
    } else {
        ("PENDING".to_string(), 0, 0, 0, "Proof pack not yet executed".to_string())
    };

    // Read rule cards
    let mut rules = Vec::new();
    let rules_dir = analysis_dir.join("rules");
    if rules_dir.exists() {
        if let Ok(entries) = fs::read_dir(rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(card) = serde_json::from_str::<RuleCard>(&content) {
                            rules.push(card);
                        }
                    }
                }
            }
        }
    }
    rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));

    // Read map.json if available
    let map_path = analysis_dir.join("map.json");
    let map_summary = if map_path.exists() {
        fs::read_to_string(map_path).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    };

    // Color banner based on verdict
    let badge_color = match verdict_str.as_str() {
        "PROVEN" => "#10b981",       // green
        "PARTLY PROVEN" => "#3b82f6", // blue
        "NOT PROVEN" => "#ef4444",    // red
        _ => "#6b7280",               // gray
    };

    let mut rules_html = String::new();
    for rule in &rules {
        let citations = rule.provenance.source_citations.join(", ");
        rules_html.push_str(&format!(
            r#"<div class="rule-card">
                <div class="rule-header">
                    <span class="rule-id">{}</span>
                    <span class="rule-module">{}</span>
                    <span class="rule-tier">{:?}</span>
                </div>
                <h4>{}</h4>
                <div class="bdd">
                    <p><strong>Given</strong> {}</p>
                    <p><strong>When</strong> {}</p>
                    <p><strong>Then</strong> {}</p>
                </div>
                <div class="rule-meta">
                    <small>Citations: {} | Verified by: {} ({:?})</small>
                </div>
            </div>"#,
            html_escape(&rule.rule_id),
            html_escape(&rule.module),
            rule.provenance.epistemic_tier,
            html_escape(&rule.title),
            html_escape(&rule.bdd.given),
            html_escape(&rule.bdd.when),
            html_escape(&rule.bdd.then),
            html_escape(&citations),
            html_escape(&rule.verification.verifier_agent),
            rule.verification.second_agent_status,
        ));
    }

    if rules_html.is_empty() {
        rules_html = "<p class='empty-state'>No BDD rule cards registered yet.</p>".to_string();
    }

    let timestamp = chrono::Utc::now().to_rfc3339();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Modernization Proof Report: {slug}</title>
    <style>
        :root {{
            --bg: #0f172a;
            --card-bg: #1e293b;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --border: #334155;
            --accent: #6366f1;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 2rem;
            line-height: 1.5;
        }}
        .container {{
            max-width: 1000px;
            margin: 0 auto;
        }}
        .header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1.5rem;
            margin-bottom: 2rem;
        }}
        .badge {{
            background: {badge_color};
            color: white;
            padding: 0.5rem 1.25rem;
            border-radius: 9999px;
            font-weight: 700;
            font-size: 1.1rem;
            letter-spacing: 0.05em;
        }}
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}
        .metric-card {{
            background: var(--card-bg);
            padding: 1.25rem;
            border-radius: 8px;
            border: 1px solid var(--border);
        }}
        .metric-value {{
            font-size: 2rem;
            font-weight: 700;
            color: var(--accent);
        }}
        .metric-label {{
            color: var(--text-muted);
            font-size: 0.875rem;
            text-transform: uppercase;
        }}
        .section {{
            margin-bottom: 2.5rem;
        }}
        .section-title {{
            font-size: 1.5rem;
            margin-bottom: 1rem;
            border-bottom: 1px solid var(--border);
            padding-bottom: 0.5rem;
        }}
        .rule-card {{
            background: var(--card-bg);
            padding: 1.25rem;
            border-radius: 8px;
            margin-bottom: 1rem;
            border-left: 4px solid var(--accent);
        }}
        .rule-header {{
            display: flex;
            gap: 0.75rem;
            align-items: center;
            margin-bottom: 0.5rem;
        }}
        .rule-id {{
            font-weight: 700;
            color: #38bdf8;
        }}
        .rule-module {{
            background: #334155;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-size: 0.75rem;
        }}
        .rule-tier {{
            background: #475569;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-size: 0.75rem;
        }}
        .bdd {{
            background: #0f172a;
            padding: 0.75rem;
            border-radius: 6px;
            margin: 0.75rem 0;
            font-family: monospace;
            font-size: 0.9rem;
        }}
        .bdd p {{ margin: 0.25rem 0; }}
        .rule-meta {{
            color: var(--text-muted);
        }}
        pre.topology {{
            background: var(--card-bg);
            padding: 1rem;
            border-radius: 8px;
            overflow-x: auto;
            font-size: 0.85rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div>
                <h1>Modernization Proof Report: {slug}</h1>
                <p style="color: var(--text-muted); margin: 0;">Generated: {timestamp}</p>
            </div>
            <div class="badge">{verdict_str}</div>
        </div>

        <div class="metrics">
            <div class="metric-card">
                <div class="metric-value">{passed_tests} / {total_tests}</div>
                <div class="metric-label">Tests Passed</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{failed_tests}</div>
                <div class="metric-label">Failed Tests</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{rules_count}</div>
                <div class="metric-label">Extracted BDD Rules</div>
            </div>
        </div>

        <div class="section">
            <h2 class="section-title">Proof Pack Verdict & Details</h2>
            <div class="metric-card">
                <p><strong>Verdict:</strong> {verdict_str}</p>
                <p><strong>Details:</strong> {details}</p>
            </div>
        </div>

        <div class="section">
            <h2 class="section-title">Extracted Business Rules (Given / When / Then)</h2>
            {rules_html}
        </div>

        <div class="section">
            <h2 class="section-title">Topology & Module Graph</h2>
            <pre class="topology"><code>{map_summary}</code></pre>
        </div>
    </div>
</body>
</html>"#,
        slug = html_escape(slug),
        timestamp = timestamp,
        verdict_str = html_escape(&verdict_str),
        total_tests = total_tests,
        passed_tests = passed_tests,
        failed_tests = failed_tests,
        rules_count = rules.len(),
        details = html_escape(&details),
        rules_html = rules_html,
        map_summary = html_escape(&map_summary),
        badge_color = badge_color,
    );

    let report_path = analysis_dir.join("REPORT.html");
    fs::write(&report_path, &html)
        .with_context(|| format!("Failed to write HTML report to {}", report_path.display()))?;

    Ok(html)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
