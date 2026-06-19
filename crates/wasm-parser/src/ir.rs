//! Intermediate representation of a Soroban contract's public interface.
//! All downstream codegen works exclusively against this IR so the parser
//! can evolve without touching templates.

use serde::{Deserialize, Serialize};

/// Top-level spec for a single contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSpec {
    /// Human-readable contract name (derived from the WASM file name or
    /// the `contractmeta` section when present).
    pub name: String,
    /// Semver string extracted from contract metadata, if present.
    pub version: Option<String>,
    /// All callable functions exposed by the contract.
    pub functions: Vec<FunctionSpec>,
    /// Named struct types referenced by functions.
    pub structs: Vec<StructSpec>,
    /// Named enum / union types referenced by functions.
    pub enums: Vec<EnumSpec>,
    /// Error code enums.
    pub error_enums: Vec<ErrorEnumSpec>,
}

/// A single callable contract function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    pub name: String,
    pub doc: Option<String>,
    pub inputs: Vec<ParamSpec>,
    pub outputs: Vec<TypeSpec>,
}

/// A function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub doc: Option<String>,
    pub type_spec: TypeSpec,
}

/// A named struct with fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructSpec {
    pub name: String,
    pub doc: Option<String>,
    pub fields: Vec<FieldSpec>,
}

/// A single struct field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSpec {
    pub name: String,
    pub doc: Option<String>,
    pub type_spec: TypeSpec,
}

/// A discriminated enum / union type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumSpec {
    pub name: String,
    pub doc: Option<String>,
    pub cases: Vec<EnumCase>,
}

/// One case within an enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumCase {
    pub name: String,
    pub doc: Option<String>,
    pub value: u32,
}

/// An error enum (integer discriminant → message).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnumSpec {
    pub name: String,
    pub doc: Option<String>,
    pub cases: Vec<ErrorCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCase {
    pub name: String,
    pub doc: Option<String>,
    pub value: u32,
}

/// The type of a parameter or return value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeSpec {
    /// `void` / unit — no value.
    Void,
    /// `bool`
    Bool,
    /// `u32`
    U32,
    /// `i32`
    I32,
    /// `u64`
    U64,
    /// `i64`
    I64,
    /// `u128`
    U128,
    /// `i128`
    I128,
    /// `u256`
    U256,
    /// `i256`
    I256,
    /// `Bytes`
    Bytes,
    /// `BytesN<N>`
    BytesN { n: u32 },
    /// `String`
    String,
    /// `Symbol`
    Symbol,
    /// `Address`
    Address,
    /// `Vec<T>`
    Vec { element: Box<TypeSpec> },
    /// `Map<K, V>`
    Map {
        key: Box<TypeSpec>,
        value: Box<TypeSpec>,
    },
    /// `Option<T>`
    Option { inner: Box<TypeSpec> },
    /// `Result<T, E>`
    Result {
        ok: Box<TypeSpec>,
        err: Box<TypeSpec>,
    },
    /// A named user-defined type (struct, enum, or error enum).
    Named { name: String },
    /// Catch-all for types not yet modelled.
    Unknown { raw: String },
}
