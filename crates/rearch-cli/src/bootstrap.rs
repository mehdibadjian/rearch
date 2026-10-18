use anyhow::{bail, Result};
use std::process::Command;

pub const CLIENTS: [&str; 4] = ["antigravity", "vscode", "copilot_cli", "claude_code"];

pub fn run_bootstrap(mode: &str) -> Result<()> {
    for client in CLIENTS {
        let args: Vec<&str> = match mode {
            "doctor" => vec![
                "-y",
                "rea-agents@latest",
                "doctor",
                "--client",
                client,
                "--json",
            ],
            "dry-run" => vec![
                "-y",
                "rea-agents@latest",
                "setup",
                "--client",
                client,
                "--dry-run",
                "--json",
            ],
            "apply" => vec![
                "-y",
                "rea-agents@latest",
                "setup",
                "--client",
                client,
                "--yes",
                "--json",
            ],
            _ => bail!("Unsupported mode: {mode}. Expected doctor, dry-run, or apply."),
        };
        println!("==> Bootstrapping REA for host [{client}] with mode [{mode}]...");
        let status = Command::new("npx").args(&args).status();
        match status {
            Ok(s) => {
                if !s.success() {
                    eprintln!(
                        "REA {mode} returned non-zero ({:?}) for client: {client}",
                        s.code()
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to execute npx for {client}: {e}. Ensure Node.js & npx are installed.");
            }
        }
    }
    Ok(())
}
