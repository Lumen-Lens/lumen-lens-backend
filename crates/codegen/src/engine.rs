//! Tera-based render engine — loads templates for a given language and
//! renders the full SDK from a [`ContractSpec`].

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use tera::Tera;
use tracing::debug;
use wasm_parser::ContractSpec;

use crate::{languages::Language, output::{RenderedFile, RenderedSdk}};

/// Options for a single codegen run.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub language: Language,
    /// Path to the directory that contains per-language template sub-dirs.
    /// Defaults to the `templates/` directory shipped alongside the binary.
    pub templates_dir: std::path::PathBuf,
    /// Whether to also emit mock-client stubs.
    pub include_mocks: bool,
}

/// Render the full SDK for `spec` using the given options.
pub fn render(spec: &ContractSpec, opts: &RenderOptions) -> Result<RenderedSdk> {
    let template_glob = opts
        .templates_dir
        .join(opts.language.template_dir())
        .join("**/*.tera");

    debug!("Loading templates from {}", template_glob.display());

    let tera = Tera::new(
        template_glob
            .to_str()
            .context("template path is not valid UTF-8")?,
    )
    .context("Failed to load Tera templates")?;

    // Build the shared template context from the IR.
    let mut context = tera::Context::new();
    context.insert("spec", spec);
    context.insert("language", &opts.language.to_string());

    let mut files = Vec::new();

    // Render each template that isn't a partial (partials start with `_`).
    for name in tera.get_template_names() {
        let file_name = std::path::Path::new(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);

        if file_name.starts_with('_') {
            continue; // skip partials
        }

        let content = tera.render(name, &context).with_context(|| {
            format!("Failed to render template '{name}'")
        })?;

        // Strip the `.tera` extension to get the output file name.
        let output_name = name
            .strip_suffix(".tera")
            .unwrap_or(name)
            .to_string();

        files.push(RenderedFile {
            path: std::path::PathBuf::from(output_name),
            content,
        });
    }

    Ok(RenderedSdk { files })
}
