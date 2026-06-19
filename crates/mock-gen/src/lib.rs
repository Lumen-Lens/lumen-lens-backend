//! `mock-gen` — thin wrapper that invokes the codegen engine with the
//! `include_mocks = true` flag and the mock-specific template overlay.

use anyhow::Result;
use codegen::{render, Language, RenderOptions, RenderedSdk};
use std::path::PathBuf;
use wasm_parser::ContractSpec;

/// Generate mock client stubs for `spec` in `language`.
///
/// `templates_dir` should point to the root templates directory that contains
/// per-language sub-directories.
pub fn generate_mocks(
    spec: &ContractSpec,
    language: Language,
    templates_dir: PathBuf,
) -> Result<RenderedSdk> {
    let opts = RenderOptions {
        language,
        templates_dir,
        include_mocks: true,
    };
    render(spec, &opts)
}
