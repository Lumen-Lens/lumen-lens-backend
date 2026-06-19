//! `diff` — detects interface changes between two versions of a contract spec
//! and classifies each change as breaking or non-breaking.

use serde::{Deserialize, Serialize};
use wasm_parser::{ir::FunctionSpec, ContractSpec};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Summary of all changes between `old` and `new` contract specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReport {
    pub contract_name: String,
    pub old_version: Option<String>,
    pub new_version: Option<String>,
    pub changes: Vec<Change>,
    pub has_breaking_changes: bool,
}

/// A single detected change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub breaking: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    FunctionAdded,
    FunctionRemoved,
    FunctionSignatureChanged,
    TypeAdded,
    TypeRemoved,
    TypeChanged,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compare `old` and `new` specs and return a [`ChangeReport`].
pub fn diff(old: &ContractSpec, new: &ContractSpec) -> ChangeReport {
    let mut changes = Vec::new();

    diff_functions(old, new, &mut changes);
    diff_structs(old, new, &mut changes);
    diff_enums(old, new, &mut changes);

    let has_breaking_changes = changes.iter().any(|c| c.breaking);

    ChangeReport {
        contract_name: new.name.clone(),
        old_version: old.version.clone(),
        new_version: new.version.clone(),
        changes,
        has_breaking_changes,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn diff_functions(old: &ContractSpec, new: &ContractSpec, changes: &mut Vec<Change>) {
    // Functions present in old but not new → removed (breaking).
    for old_fn in &old.functions {
        if !new.functions.iter().any(|f| f.name == old_fn.name) {
            changes.push(Change {
                kind: ChangeKind::FunctionRemoved,
                breaking: true,
                description: format!("Function `{}` was removed.", old_fn.name),
            });
        }
    }

    // Functions present in new but not old → added (non-breaking).
    for new_fn in &new.functions {
        if !old.functions.iter().any(|f| f.name == new_fn.name) {
            changes.push(Change {
                kind: ChangeKind::FunctionAdded,
                breaking: false,
                description: format!("Function `{}` was added.", new_fn.name),
            });
        }
    }

    // Functions in both — check for signature changes.
    for new_fn in &new.functions {
        if let Some(old_fn) = old.functions.iter().find(|f| f.name == new_fn.name) {
            check_function_signature(old_fn, new_fn, changes);
        }
    }
}

fn check_function_signature(old: &FunctionSpec, new: &FunctionSpec, changes: &mut Vec<Change>) {
    // Parameter count change.
    if old.inputs.len() != new.inputs.len() {
        changes.push(Change {
            kind: ChangeKind::FunctionSignatureChanged,
            breaking: true,
            description: format!(
                "Function `{}` parameter count changed from {} to {}.",
                new.name,
                old.inputs.len(),
                new.inputs.len()
            ),
        });
        return;
    }

    // Parameter type/name changes.
    for (i, (op, np)) in old.inputs.iter().zip(new.inputs.iter()).enumerate() {
        if op.name != np.name {
            changes.push(Change {
                kind: ChangeKind::FunctionSignatureChanged,
                breaking: true,
                description: format!(
                    "Function `{}` param {} renamed from `{}` to `{}`.",
                    new.name, i, op.name, np.name
                ),
            });
        }
        let old_type = serde_json::to_string(&op.type_spec).unwrap_or_default();
        let new_type = serde_json::to_string(&np.type_spec).unwrap_or_default();
        if old_type != new_type {
            changes.push(Change {
                kind: ChangeKind::FunctionSignatureChanged,
                breaking: true,
                description: format!(
                    "Function `{}` param `{}` type changed from {} to {}.",
                    new.name, np.name, old_type, new_type
                ),
            });
        }
    }

    // Return type changes.
    let old_outs = serde_json::to_string(&old.outputs).unwrap_or_default();
    let new_outs = serde_json::to_string(&new.outputs).unwrap_or_default();
    if old_outs != new_outs {
        changes.push(Change {
            kind: ChangeKind::FunctionSignatureChanged,
            breaking: true,
            description: format!("Function `{}` return type changed.", new.name),
        });
    }
}

fn diff_structs(old: &ContractSpec, new: &ContractSpec, changes: &mut Vec<Change>) {
    for s in &old.structs {
        if !new.structs.iter().any(|x| x.name == s.name) {
            changes.push(Change {
                kind: ChangeKind::TypeRemoved,
                breaking: true,
                description: format!("Struct `{}` was removed.", s.name),
            });
        }
    }
    for s in &new.structs {
        if !old.structs.iter().any(|x| x.name == s.name) {
            changes.push(Change {
                kind: ChangeKind::TypeAdded,
                breaking: false,
                description: format!("Struct `{}` was added.", s.name),
            });
        }
    }
}

fn diff_enums(old: &ContractSpec, new: &ContractSpec, changes: &mut Vec<Change>) {
    for e in &old.enums {
        if !new.enums.iter().any(|x| x.name == e.name) {
            changes.push(Change {
                kind: ChangeKind::TypeRemoved,
                breaking: true,
                description: format!("Enum `{}` was removed.", e.name),
            });
        }
    }
    for e in &new.enums {
        if !old.enums.iter().any(|x| x.name == e.name) {
            changes.push(Change {
                kind: ChangeKind::TypeAdded,
                breaking: false,
                description: format!("Enum `{}` was added.", e.name),
            });
        }
    }
}
