use anyhow::Result;
use clap::{Parser, Subcommand};
use rearch_cli::{bootstrap, commands, loop_runner};

#[derive(Parser)]
#[command(
    name = "rearch",
    version = "1.1",
    about = "Autonomous REA & Code-Modernization Engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bootstrap {
        #[arg(long, default_value = "doctor")]
        mode: String,
    },
    Intent {
        slug: String,
    },
    Preflight {
        slug: String,
        #[arg(long)]
        source: Option<String>,
    },
    Assess {
        slug: String,
    },
    Map {
        slug: String,
    },
    ExtractRules {
        slug: String,
    },
    Review {
        slug: String,
    },
    Brief {
        slug: String,
    },
    Sign {
        slug: String,
    },
    Uplift {
        slug: String,
    },
    Transform {
        slug: String,
    },
    Reimagine {
        slug: String,
    },
    Verify {
        slug: String,
    },
    LoopStep {
        slug: String,
        #[arg(long)]
        window: Option<String>,
        #[arg(index = 2)]
        positional_window: Option<String>,
        #[arg(long, default_value_t = 15)]
        max_iterations: u32,
    },
    ProofPack {
        slug: String,
    },
    Harden {
        slug: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Bootstrap { mode } => bootstrap::run_bootstrap(&mode)?,
        Commands::Intent { slug } => commands::run_intent(&slug)?,
        Commands::Preflight { slug, source } => commands::run_preflight(&slug, source.as_deref())?,
        Commands::Assess { slug } => commands::run_assess(&slug)?,
        Commands::Map { slug } => commands::run_map(&slug)?,
        Commands::ExtractRules { slug } => commands::run_extract_rules(&slug)?,
        Commands::Review { slug } => commands::run_review(&slug)?,
        Commands::Brief { slug } => commands::run_brief(&slug)?,
        Commands::Sign { slug } => commands::run_sign(&slug)?,
        Commands::Uplift { slug } => commands::run_build_stage(&slug, "uplift")?,
        Commands::Transform { slug } => commands::run_build_stage(&slug, "transform")?,
        Commands::Reimagine { slug } => commands::run_build_stage(&slug, "reimagine")?,
        Commands::Verify { slug } | Commands::ProofPack { slug } => {
            let report = rearch_proof::verifier::run_proof_pack(&slug)?;
            rearch_proof::report::render_html_report(&slug)?;
            println!(
                "Proof Pack Verdict: {:?} ({})",
                report.verdict, report.details
            );
        }
        Commands::LoopStep {
            slug,
            window,
            positional_window,
            max_iterations,
        } => {
            let win = window
                .or(positional_window)
                .unwrap_or_else(|| "discover".to_string());
            loop_runner::execute_loop_step(&slug, &win, max_iterations)?;
        }
        Commands::Harden { slug } => commands::run_harden(&slug)?,
    }
    Ok(())
}
