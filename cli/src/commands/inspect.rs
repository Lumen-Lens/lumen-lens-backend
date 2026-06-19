//! `lumenlens inspect` — dump the parsed spec as JSON.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use wasm_parser::parse_wasm;

#[derive(Args)]
pub struct InspectArgs {
    /// Path to the compiled Soroban WASM file.
    #[arg(short, long, value_name = "FILE")]
    pub wasm: PathBuf,
}

pub fn run(args: InspectArgs) -> Result<()> {
    let wasm_bytes = std::fs::read(&args.wasm)
        .with_context(|| format!("Cannot read WASM: {}", args.wasm.display()))?;
    let name_hint = args
        .wasm
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");
    let spec = parse_wasm(&wasm_bytes, name_hint)?;
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
