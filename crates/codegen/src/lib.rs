//! `codegen` — drives the Tera template engine to produce per-language SDKs.

pub mod engine;
pub mod languages;
pub mod output;

pub use engine::{render, RenderOptions};
pub use languages::Language;
pub use output::RenderedSdk;
