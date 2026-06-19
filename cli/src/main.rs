//! Lumen Lens CLI entry point.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

mod commands;

#[derive(Parser)]
#[command(
    name = "lumenlens",
    version,
    about = "Generate typed cross-language SDKs from Soroban contract WASMs.",
    long_about = None,
)]
struct Cli {
    /// Increase log verbosity (set RUST_LOG for fine-grained control).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an SDK from a local WASM file.
    Generate(commands::generate::GenerateArgs),

    /// Diff two WASM files and report breaking changes.
    Diff(commands::diff::DiffArgs),

    /// Generate a mock client for local frontend development.
    Mock(commands::mock::MockArgs),

    /// Print the parsed contract spec as JSON (useful for debugging).
    Inspect(commands::inspect::InspectArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing subscriber.
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .init();

    match cli.command {
        Commands::Generate(args) => commands::generate::run(args),
        Commands::Diff(args) => commands::diff::run(args),
        Commands::Mock(args) => commands::mock::run(args),
        Commands::Inspect(args) => commands::inspect::run(args),
    }
}
