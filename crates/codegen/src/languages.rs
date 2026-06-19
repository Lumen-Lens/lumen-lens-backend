//! Supported target languages and their template/output configuration.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "typescript" | "ts" => Ok(Self::TypeScript),
            "go" => Ok(Self::Go),
            "python" | "py" => Ok(Self::Python),
            "dart" => Ok(Self::Dart),
            other => Err(format!("unknown language: {other}")),
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
