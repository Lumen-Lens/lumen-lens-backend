//! Types representing a rendered SDK ready to write to disk.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A single generated file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFile {
    /// Relative path within the output directory (e.g. `src/index.ts`).
    pub path: PathBuf,
    /// UTF-8 contents.
    pub content: String,
}

/// All files that make up a generated SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedSdk {
    pub files: Vec<RenderedFile>,
}

impl RenderedSdk {
    /// Write all files to `output_dir`, creating directories as needed.
    pub fn write_to_disk(&self, output_dir: &std::path::Path) -> anyhow::Result<()> {
        for file in &self.files {
            let dest = output_dir.join(&file.path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, &file.content)?;
            tracing::info!("Wrote {}", dest.display());
        }
        Ok(())
    }
}
