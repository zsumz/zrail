//! Macro authority is reasoned, non-overlapping, and optionally definition-narrowed.

use std::collections::BTreeSet;

use crate::contract::{
    Contract, MacroExpansionMode, validate_dependencies, validate_limits::ValidationErrors,
    validate_paths::validate_repository_literal, validate_sets::require_reason,
};

use super::valid_rust_path;

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    if contract.source.rust.macros.mode == MacroExpansionMode::Allow
        && !contract.source.rust.macros.allow.is_empty()
    {
        errors.push("source.rust.macros.allow requires macros.mode = \"deny-unreviewed\"".into());
    }
    let mut names = BTreeSet::new();
    for allowed in &contract.source.rust.macros.allow {
        require_reason("macro expansion", &allowed.name, &allowed.reason, errors);
        if !valid_rust_path(&allowed.name) {
            errors.push(format!(
                "allowed macro expansion name must be a Rust path: {:?}",
                allowed.name
            ));
        }
        if !names.insert(allowed.name.as_str()) {
            errors.push(format!(
                "duplicate macro expansion allowance {:?}",
                allowed.name
            ));
        }
        if let Some(path) = &allowed.definition {
            validate_repository_literal(path, errors);
        }
        if let Some(source) = &allowed.source {
            validate_dependencies::validate_source(source, &allowed.name, errors);
            if allowed.definition.is_some() {
                errors.push(format!(
                    "macro expansion allowance {:?} may not combine a repository definition with external source identity",
                    allowed.name
                ));
            }
        }
    }
}
