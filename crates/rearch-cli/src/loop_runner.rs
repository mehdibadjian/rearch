use anyhow::{bail, Result};
use rearch_core::gates::{
    verify_gate_1_locked_at, verify_gate_3_signed_at, verify_legacy_unmodified_at,
};
use rearch_core::models::ProofVerdict;
use rearch_proof::{report::render_html_report_at, verifier::run_proof_pack_at};
use std::{fs, path::Path};

pub fn execute_loop_step(slug: &str, window: &str, max_iterations: u32) -> Result<()> {
    execute_loop_step_at(Path::new("."), slug, window, max_iterations)
}

pub fn execute_loop_step_at(
    base_dir: &Path,
    slug: &str,
    window: &str,
    max_iterations: u32,
) -> Result<()> {
    let analysis_dir = base_dir.join("analysis").join(slug);
    verify_legacy_unmodified_at(base_dir, slug)?;
    verify_gate_1_locked_at(base_dir, slug)?;

    let progress_path = analysis_dir.join("LOOP_PROGRESS.md");
    let progress = fs::read_to_string(&progress_path).unwrap_or_default();
    let lines: Vec<&str> = progress.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.len() as u32 >= max_iterations {
        bail!("LOOP HALT: Max iteration cap ({max_iterations}) reached for {slug}.");
    }

    // Stall detector: halt if the last 2 iterations produced identical progress signatures
    if lines.len() >= 2 && lines[lines.len() - 1] == lines[lines.len() - 2] {
        bail!("LOOP HALT: Stall detected (2 consecutive identical outcomes). Pivot required.");
    }

    match window {
        "discover" => {
            let evidence =
                fs::read_to_string(analysis_dir.join("EVIDENCE_STATE.md")).unwrap_or_default();
            let open_unknowns = evidence.lines().filter(|l| l.contains("`UNKNOWN`")).count();
            let _ = render_html_report_at(base_dir, slug)?;
            if open_unknowns == 0 && evidence.contains("| **Q-") {
                println!("GOAL MET [Window A]: 0 UNKNOWNs remain in EVIDENCE_STATE.md.");
                return Ok(());
            }
            bail!("CONTINUE_LOOP: {open_unknowns} UNKNOWN items remain in EVIDENCE_STATE.md.");
        }
        "build" => {
            verify_gate_3_signed_at(base_dir, slug)?;
            let report = run_proof_pack_at(base_dir, slug)?;
            let _ = render_html_report_at(base_dir, slug)?;
            match report.verdict {
                ProofVerdict::Proven | ProofVerdict::PartlyProven => {
                    println!(
                        "GOAL MET [Window B]: Verdict is {:?} ({})",
                        report.verdict, report.details
                    );
                    Ok(())
                }
                ProofVerdict::NotProven => {
                    bail!("CONTINUE_LOOP: Verdict is NOT PROVEN ({})", report.details);
                }
            }
        }
        _ => bail!("Unknown loop window: {window}. Expected 'discover' or 'build'."),
    }
}
