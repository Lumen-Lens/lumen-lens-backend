//! Reads the `contractspecv0` custom section from a Soroban WASM binary and
//! converts the XDR-encoded `SCSpecEntry` stream into a [`ContractSpec`].

use anyhow::{bail, Context, Result};
use tracing::debug;
use wasmparser::{Parser, Payload};

use crate::ir::{
    ContractSpec, EnumCase, EnumSpec, ErrorCase, ErrorEnumSpec, FieldSpec, FunctionSpec, ParamSpec,
    StructSpec, TypeSpec,
};

const SPEC_SECTION: &str = "contractspecv0";
const META_SECTION: &str = "contractmetav0";

/// Parse a Soroban WASM binary and return its contract spec IR.
///
/// `name_hint` is used as the contract name when the WASM metadata does not
/// embed one (e.g. pass the file stem).
pub fn parse_wasm(wasm_bytes: &[u8], name_hint: &str) -> Result<ContractSpec> {
    let mut spec_bytes: Option<Vec<u8>> = None;
    let mut meta_bytes: Option<Vec<u8>> = None;

    for payload in Parser::new(0).parse_all(wasm_bytes) {
        let payload = payload.context("WASM parse error")?;
        if let Payload::CustomSection(reader) = payload {
            match reader.name() {
                SPEC_SECTION => {
                    debug!("Found contractspecv0 section ({} bytes)", reader.data().len());
                    spec_bytes = Some(reader.data().to_vec());
                }
                META_SECTION => {
                    meta_bytes = Some(reader.data().to_vec());
                }
                _ => {}
            }
        }
    }

    let spec_bytes = spec_bytes.context(
        "No `contractspecv0` custom section found — is this a Soroban contract?",
    )?;

    let (name, version) = parse_metadata(meta_bytes.as_deref(), name_hint);
    let entries = decode_spec_entries(&spec_bytes)?;

    build_ir(name, version, entries)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_metadata(bytes: Option<&[u8]>, hint: &str) -> (String, Option<String>) {
    // The contractmetav0 section is a sequence of SCMetaEntry XDR values.
    // For now we just use the name hint and skip deep XDR parsing of meta —
    // a future PR can wire up stellar-xdr properly here.
    let _ = bytes; // reserved for future use
    (hint.to_string(), None)
}

/// Decode the raw bytes of the `contractspecv0` section into XDR SCSpecEntry
/// values.  We use stellar-xdr's `Read` trait to consume the stream.
fn decode_spec_entries(bytes: &[u8]) -> Result<Vec<stellar_xdr::curr::ScSpecEntry>> {
    use stellar_xdr::curr::{Limited, Limits, ReadXdr, ScSpecEntry};
    use std::io::Cursor;

    let mut entries = Vec::new();
    let mut cursor = Limited::new(Cursor::new(bytes), Limits::none());

    loop {
        match ScSpecEntry::read_xdr(&mut cursor) {
            Ok(entry) => entries.push(entry),
            Err(stellar_xdr::curr::Error::Io(_)) => break, // EOF
            Err(e) => bail!("XDR decode error: {e}"),
        }
    }

    debug!("Decoded {} SCSpecEntry values", entries.len());
    Ok(entries)
}

fn build_ir(
    name: String,
    version: Option<String>,
    entries: Vec<stellar_xdr::curr::ScSpecEntry>,
) -> Result<ContractSpec> {
    use stellar_xdr::curr::ScSpecEntry;

    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut error_enums = Vec::new();

    for entry in entries {
        match entry {
            ScSpecEntry::FunctionV0(f) => functions.push(convert_function(f)?),
            ScSpecEntry::UdtStructV0(s) => structs.push(convert_struct(s)?),
            ScSpecEntry::UdtUnionV0(u) => enums.push(convert_union(u)?),
            ScSpecEntry::UdtEnumV0(e) => enums.push(convert_enum(e)?),
            ScSpecEntry::UdtErrorEnumV0(e) => error_enums.push(convert_error_enum(e)?),
        }
    }

    Ok(ContractSpec {
        name,
        version,
        functions,
        structs,
        enums,
        error_enums,
    })
}

// ---------------------------------------------------------------------------
// Converters — SCSpecEntry subtypes → IR types
// ---------------------------------------------------------------------------

fn convert_function(f: stellar_xdr::curr::ScSpecFunctionV0) -> Result<FunctionSpec> {
    use stellar_xdr::curr::StringM;

    let name = f.name.to_utf8_string().context("function name UTF-8")?;
    let doc = xdr_doc(&f.doc);
    let inputs = f
        .inputs
        .iter()
        .map(|p| {
            Ok(ParamSpec {
                name: p.name.to_utf8_string().context("param name")?,
                doc: xdr_doc(&p.doc),
                type_spec: convert_type(&p.type_)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let outputs = f
        .outputs
        .iter()
        .map(convert_type)
        .collect::<Result<Vec<_>>>()?;

    Ok(FunctionSpec {
        name,
        doc,
        inputs,
        outputs,
    })
}

fn convert_struct(s: stellar_xdr::curr::ScSpecUdtStructV0) -> Result<StructSpec> {
    let name = s.name.to_utf8_string().context("struct name")?;
    let doc = xdr_doc(&s.doc);
    let fields = s
        .fields
        .iter()
        .map(|f| {
            Ok(FieldSpec {
                name: f.name.to_utf8_string().context("field name")?,
                doc: xdr_doc(&f.doc),
                type_spec: convert_type(&f.type_)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(StructSpec { name, doc, fields })
}

fn convert_union(u: stellar_xdr::curr::ScSpecUdtUnionV0) -> Result<EnumSpec> {
    use stellar_xdr::curr::ScSpecUdtUnionCaseV0;

    let name = u.name.to_utf8_string().context("union name")?;
    let doc = xdr_doc(&u.doc);
    let cases = u
        .cases
        .iter()
        .enumerate()
        .map(|(i, c)| match c {
            ScSpecUdtUnionCaseV0::VoidV0(v) => Ok(EnumCase {
                name: v.name.to_utf8_string().context("union case name")?,
                doc: xdr_doc(&v.doc),
                value: i as u32,
            }),
            ScSpecUdtUnionCaseV0::TupleV0(t) => Ok(EnumCase {
                name: t.name.to_utf8_string().context("union tuple name")?,
                doc: xdr_doc(&t.doc),
                value: i as u32,
            }),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(EnumSpec { name, doc, cases })
}

fn convert_enum(e: stellar_xdr::curr::ScSpecUdtEnumV0) -> Result<EnumSpec> {
    let name = e.name.to_utf8_string().context("enum name")?;
    let doc = xdr_doc(&e.doc);
    let cases = e
        .cases
        .iter()
        .map(|c| {
            Ok(EnumCase {
                name: c.name.to_utf8_string().context("enum case name")?,
                doc: xdr_doc(&c.doc),
                value: c.value,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(EnumSpec { name, doc, cases })
}

fn convert_error_enum(e: stellar_xdr::curr::ScSpecUdtErrorEnumV0) -> Result<ErrorEnumSpec> {
    let name = e.name.to_utf8_string().context("error enum name")?;
    let doc = xdr_doc(&e.doc);
    let cases = e
        .cases
        .iter()
        .map(|c| {
            Ok(ErrorCase {
                name: c.name.to_utf8_string().context("error case name")?,
                doc: xdr_doc(&c.doc),
                value: c.value,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ErrorEnumSpec { name, doc, cases })
}

fn convert_type(t: &stellar_xdr::curr::ScSpecTypeDef) -> Result<TypeSpec> {
    use stellar_xdr::curr::ScSpecTypeDef as T;

    Ok(match t {
        T::Val => TypeSpec::Unknown { raw: "Val".into() },
        T::Bool => TypeSpec::Bool,
        T::Void => TypeSpec::Void,
        T::Error => TypeSpec::Unknown { raw: "Error".into() },
        T::U32 => TypeSpec::U32,
        T::I32 => TypeSpec::I32,
        T::U64 => TypeSpec::U64,
        T::I64 => TypeSpec::I64,
        T::Timepoint => TypeSpec::U64,
        T::Duration => TypeSpec::U64,
        T::U128 => TypeSpec::U128,
        T::I128 => TypeSpec::I128,
        T::U256 => TypeSpec::U256,
        T::I256 => TypeSpec::I256,
        T::Bytes => TypeSpec::Bytes,
        T::String => TypeSpec::String,
        T::Symbol => TypeSpec::Symbol,
        T::Address => TypeSpec::Address,
        T::Option(o) => TypeSpec::Option {
            inner: Box::new(convert_type(&o.value_type)?),
        },
        T::Result(r) => TypeSpec::Result {
            ok: Box::new(convert_type(&r.ok_type)?),
            err: Box::new(convert_type(&r.error_type)?),
        },
        T::Vec(v) => TypeSpec::Vec {
            element: Box::new(convert_type(&v.element_type)?),
        },
        T::Map(m) => TypeSpec::Map {
            key: Box::new(convert_type(&m.key_type)?),
            value: Box::new(convert_type(&m.value_type)?),
        },
        T::BytesN(b) => TypeSpec::BytesN { n: b.n },
        T::Udt(u) => TypeSpec::Named {
            name: u.name.to_utf8_string().unwrap_or_default(),
        },
        _ => TypeSpec::Unknown {
            raw: format!("{t:?}"),
        },
    })
}

fn xdr_doc(doc: &stellar_xdr::curr::StringM<1024>) -> Option<String> {
    let s = doc.to_utf8_string().unwrap_or_default();
    if s.is_empty() { None } else { Some(s) }
}
