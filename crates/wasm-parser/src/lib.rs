//! `wasm-parser` — extracts the Soroban contract spec from a compiled WASM binary.
//!
//! The Soroban SDK embeds XDR-encoded `SCSpecEntry` structs in a custom WASM
//! section named `contractspecv0`. This crate reads that section and decodes
//! the entries into an ergonomic IR used by the rest of Lumen Lens.

pub mod ir;
pub mod reader;

pub use ir::ContractSpec;
pub use reader::parse_wasm;
