//! `lumenlens mock` — generate a mock client stub.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use codegen::Language;
use wasm_parser::parse_wasm;

#[derive(Args)]
pub struct MockArgs {
    /// Path to the Soroban WASM file (or use --spec-json to skip parsing).
    #[arg(short, long, value_name = "FILE")]
    pub wasm: PathBuf,

    /// Target language: typescript | go | python | dart.
    #[arg(short, long, default_value = "typescript")]
    pub language: String,

    /// Output directory for generated files.
    #[arg(short, long, default_value = "./mock-out")]
    pub output: PathBuf,

    /// Directory containing Tera templates.
    #[arg(long, default_value = "./templates")]
    pub templates_dir: PathBuf,
}

pub fn run(args: MockArgs) -> Result<()> {
    let lang = args
        .language
        .parse::<Language>()
        .map_err(|e| anyhow::anyhow!(e))
        .with_context(|| format!("Unknown language: {}", args.language))?;

    let wasm_bytes = std::fs::read(&args.wasm)
        .with_context(|| format!("Cannot read WASM: {}", args.wasm.display()))?;
    let name_hint = args
        .wasm
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");
    let spec = parse_wasm(&wasm_bytes, name_hint)?;

    let sdk = mock_gen::generate_mocks(&spec, lang.clone(), args.templates_dir)?;
    sdk.write_to_disk(&args.output)?;

    println!(
        "✅  Generated {} mock client → {}",
        lang,
        args.output.display()
    );
    Ok(())
}
