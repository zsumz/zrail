//! Macro authority is reasoned, non-overlapping, and locally content-bindable.

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
        let local = allowed.name.starts_with("local::");
        match (&allowed.definition, local) {
            (Some(path), true) => validate_repository_literal(path, errors),
            (None, true) => errors.push(format!(
                "local macro expansion allowance {:?} requires an exact definition path",
                allowed.name
            )),
            (Some(_), false) => errors.push(format!(
                "external macro expansion allowance {:?} may not declare a local definition path",
                allowed.name
            )),
            (None, false) => {}
        }
        if let Some(source) = &allowed.source {
            validate_dependencies::validate_source(source, &allowed.name, errors);
            if local {
                errors.push(format!(
                    "local macro expansion allowance {:?} may not declare external source identity",
                    allowed.name
                ));
            }
        }
    }
}
