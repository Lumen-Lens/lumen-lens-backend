//! Supported target languages and their template/output configuration.

use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    TypeScript,
    Go,
    Python,
    Dart,
}

impl Language {
    /// Returns the directory name used to locate templates for this language.
    pub fn template_dir(&self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::Go => "go",
            Self::Python => "python",
            Self::Dart => "dart",
        }
    }

    /// Default file extension for generated source files.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::TypeScript => "ts",
            Self::Go => "go",
            Self::Python => "py",
            Self::Dart => "dart",
        }
    }

    /// Parse from a CLI-style string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "typescript" | "ts" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "python" | "py" => Some(Self::Python),
            "dart" => Some(Self::Dart),
            _ => None,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeScript => write!(f, "typescript"),
            Self::Go => write!(f, "go"),
            Self::Python => write!(f, "python"),
            Self::Dart => write!(f, "dart"),
        }
    }
}
