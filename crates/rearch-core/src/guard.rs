use anyhow::{bail, Result};
use std::{fs, path::Path};

pub fn scan_untrusted_output(
    base_dir: &Path,
    slug: &str,
    tool_name: &str,
    raw_output: &str,
) -> Result<()> {
    let injection_patterns = [
        "Ignore previous instructions",
        "System prompt:",
        "You are now a",
        "Disregard all prior rules",
        "Disregard previous instructions",
        "Disregard all previous instructions",
        "Override system prompt",
    ];

    for pattern in &injection_patterns {
        if raw_output.to_lowercase().contains(&pattern.to_lowercase()) {
            let log_dir = base_dir.join("analysis").join(slug);
            fs::create_dir_all(&log_dir)?;
            let finding = format!(
                r#"{{"timestamp": "{}", "tool": "{}", "pattern": "{}", "message": "PROMPT INJECTION DETECTED in tool {}: matched pattern '{}'"}}"#,
                chrono::Utc::now().to_rfc3339(),
                tool_name,
                pattern,
                tool_name,
                pattern
            );
            let findings_path = log_dir.join("security_findings.jsonl");
            let mut existing = fs::read_to_string(&findings_path).unwrap_or_default();
            existing.push_str(&finding);
            existing.push('\n');
            fs::write(findings_path, existing)?;

            bail!(
                "SECURITY GUARD: Potential prompt injection payload detected from REA tool output. Logged to security_findings.jsonl."
            );
        }
    }
    Ok(())
}

pub fn scan_untrusted_output_cwd(slug: &str, tool_name: &str, raw_output: &str) -> Result<()> {
    scan_untrusted_output(Path::new("."), slug, tool_name, raw_output)
}
