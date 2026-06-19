//! `lumenlens generate` — parse a WASM and emit an SDK.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

use codegen::{Language, RenderOptions};
use wasm_parser::parse_wasm;

#[derive(Args)]
pub struct GenerateArgs {
    /// Path to the compiled Soroban WASM file.
    #[arg(short, long, value_name = "FILE")]
    pub wasm: PathBuf,

    /// Target language: typescript | go | python | dart.
    #[arg(short, long, default_value = "typescript")]
    pub language: String,

    /// Output directory for generated files.
    #[arg(short, long, default_value = "./sdk-out")]
    pub output: PathBuf,

    /// Directory containing Tera templates.
    #[arg(long, default_value = "./templates")]
    pub templates_dir: PathBuf,
}

pub fn run(args: GenerateArgs) -> Result<()> {
    let lang = Language::from_str(&args.language)
        .with_context(|| format!("Unknown language: {}", args.language))?;

    let wasm_bytes = std::fs::read(&args.wasm)
        .with_context(|| format!("Cannot read WASM file: {}", args.wasm.display()))?;

    let name_hint = args
        .wasm
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");

    let spec = parse_wasm(&wasm_bytes, name_hint)?;

    tracing::info!(
        "Parsed contract '{}' with {} functions",
        spec.name,
        spec.functions.len()
    );

    let opts = RenderOptions {
        language: lang.clone(),
        templates_dir: args.templates_dir,
        include_mocks: false,
    };

    let sdk = codegen::render(&spec, &opts)?;
    sdk.write_to_disk(&args.output)?;

    println!(
        "✅  Generated {} SDK → {}",
        lang,
        args.output.display()
    );
    Ok(())
}
