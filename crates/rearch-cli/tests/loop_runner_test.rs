use std::fs;
use tempfile::tempdir;

// We'll test loop_runner logic
#[test]
fn test_loop_runner_stall_detection() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    fs::create_dir_all(&analysis_dir).unwrap();

    // Gate 1 approved
    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true}"#,
    )
    .unwrap();

    // Two identical lines in LOOP_PROGRESS.md to simulate stall
    let progress_path = analysis_dir.join("LOOP_PROGRESS.md");
    fs::write(
        &progress_path,
        "outcome: examined auth.rs (unknown: 3)\noutcome: examined auth.rs (unknown: 3)\n",
    )
    .unwrap();

    let res = rearch_cli::loop_runner::execute_loop_step_at(temp.path(), "sample", "discover", 15);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("Stall detected"));
}

#[test]
fn test_loop_runner_max_iteration_cap() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    fs::create_dir_all(&analysis_dir).unwrap();

    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true}"#,
    )
    .unwrap();

    let progress_path = analysis_dir.join("LOOP_PROGRESS.md");
    fs::write(&progress_path, "Iter 1\nIter 2\nIter 3\nIter 4\nIter 5\n").unwrap();

    let res = rearch_cli::loop_runner::execute_loop_step_at(temp.path(), "sample", "discover", 5);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("Max iteration cap (5) reached"));
}

#[test]
fn test_loop_runner_discover_window_continues_when_unknowns_remain() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    fs::create_dir_all(&analysis_dir).unwrap();

    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true}"#,
    )
    .unwrap();

    fs::write(
        analysis_dir.join("EVIDENCE_STATE.md"),
        "| **Q-01** | `UNKNOWN` | How does auth work? |\n",
    )
    .unwrap();

    let res = rearch_cli::loop_runner::execute_loop_step_at(temp.path(), "sample", "discover", 15);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("CONTINUE_LOOP: 1 UNKNOWN items remain"));
}

#[test]
fn test_loop_runner_discover_window_goal_met_when_zero_unknowns() {
    let temp = tempdir().unwrap();
    let analysis_dir = temp.path().join("analysis/sample");
    fs::create_dir_all(&analysis_dir).unwrap();

    fs::write(
        analysis_dir.join("preflight.json"),
        r#"{"gate_1_approved": true}"#,
    )
    .unwrap();

    fs::write(
        analysis_dir.join("EVIDENCE_STATE.md"),
        "| **Q-01** | `OBSERVATION` | Auth uses JWT tokens |\n",
    )
    .unwrap();

    let res = rearch_cli::loop_runner::execute_loop_step_at(temp.path(), "sample", "discover", 15);
    assert!(res.is_ok());
}
