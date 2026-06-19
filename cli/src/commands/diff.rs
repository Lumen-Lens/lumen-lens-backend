//! `lumenlens diff` — compare two WASM versions and report changes.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use wasm_parser::parse_wasm;

#[derive(Args)]
pub struct DiffArgs {
    /// Path to the OLD WASM file.
    #[arg(long, value_name = "FILE")]
    pub old: PathBuf,

    /// Path to the NEW WASM file.
    #[arg(long, value_name = "FILE")]
    pub new: PathBuf,

    /// Output format: text (default) or json.
    #[arg(long, default_value = "text")]
    pub format: String,
}

pub fn run(args: DiffArgs) -> Result<()> {
    let old_bytes = std::fs::read(&args.old)
        .with_context(|| format!("Cannot read old WASM: {}", args.old.display()))?;
    let new_bytes = std::fs::read(&args.new)
        .with_context(|| format!("Cannot read new WASM: {}", args.new.display()))?;

    let old_name = args
        .old
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("old");
    let new_name = args
        .new
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("new");

    let old_spec = parse_wasm(&old_bytes, old_name)?;
    let new_spec = parse_wasm(&new_bytes, new_name)?;

    let report = diff::diff(&old_spec, &new_spec);

    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&report)?),
        _ => {
            println!("Contract: {}", report.contract_name);
            println!(
                "Versions: {} → {}",
                report.old_version.as_deref().unwrap_or("unknown"),
                report.new_version.as_deref().unwrap_or("unknown")
            );
            println!();
            if report.changes.is_empty() {
                println!("✅  No interface changes detected.");
            } else {
                for c in &report.changes {
                    let icon = if c.breaking { "❌ BREAKING" } else { "✅ " };
                    println!("  {icon}  {}", c.description);
                }
                println!();
                if report.has_breaking_changes {
                    println!("⚠️  Breaking changes detected — bump the major version.");
                }
            }
        }
    }
    Ok(())
}
