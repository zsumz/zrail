//! Item-producing macro authority is scoped, reasoned, and provenance-aware.

use std::collections::BTreeSet;

use crate::contract::{Contract, MacroBindingMode, validate_dependencies};

use super::{
    super::{
        validate_limits::ValidationErrors,
        validate_paths::{validate_repository_literal, validate_repository_pattern},
        validate_sets::require_reason,
    },
    valid_rust_path,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut identities = BTreeSet::new();
    for allowance in &contract.source.rust.item_macros {
        validate_selector(allowance, errors);
        require_reason("item macro", &allowance.name, &allowance.reason, errors);
        if !valid_rust_path(&allowance.name) {
            errors.push(format!(
                "item macro name must be a Rust path: {:?}",
                allowance.name
            ));
        }
        validate_binding(allowance, errors);
        if !identities.insert(identity(allowance)) {
            errors.push(format!(
                "duplicate item macro authority for {} in {}",
                allowance.name,
                selector_name(allowance)
            ));
        }
    }
}

fn validate_selector(allowance: &crate::ItemMacroContract, errors: &mut ValidationErrors) {
    if allowance.path.is_some() && !allowance.within.is_empty() {
        errors.push(format!(
            "item macro authority {:?} may not combine path and within",
            allowance.name
        ));
    }
    if let Some(path) = &allowance.path {
        validate_repository_literal(path, errors);
        if !std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        {
            errors.push(format!("item macro path must name .rs source: {path:?}"));
        }
    }
    let mut patterns = BTreeSet::new();
    for pattern in &allowance.within {
        validate_repository_pattern(pattern, errors);
        if !patterns.insert(pattern) {
            errors.push(format!(
                "item macro authority {:?} contains duplicate within selector {pattern:?}",
                allowance.name
            ));
        }
    }
}

fn validate_binding(allowance: &crate::ItemMacroContract, errors: &mut ValidationErrors) {
    if let Some(source) = &allowance.source {
        validate_dependencies::validate_source(source, &allowance.name, errors);
    }
    if allowance.source.is_some() && allowance.binding != Some(MacroBindingMode::Exact) {
        errors.push(format!(
            "item macro authority {:?} requires resolution = \"exact\" when source is set",
            allowance.name
        ));
    }
    let Some(manifest) = &allowance.manifest else {
        return;
    };
    validate_repository_literal(manifest, errors);
    if allowance.path.is_none() || !allowance.within.is_empty() {
        errors.push(format!(
            "item macro authority {:?} with an exact manifest requires one exact path",
            allowance.name
        ));
    }
    if allowance.binding != Some(MacroBindingMode::Exact) {
        errors.push(format!(
            "item macro authority {:?} with an exact manifest requires resolution = \"exact\"",
            allowance.name
        ));
    }
}

fn identity(allowance: &crate::ItemMacroContract) -> String {
    let mut within = allowance.within.clone();
    within.sort();
    format!(
        "{}:{}:{}",
        allowance.name,
        allowance.path.as_deref().unwrap_or("<name>"),
        within.join(",")
    )
}

fn selector_name(allowance: &crate::ItemMacroContract) -> String {
    allowance.path.as_ref().map_or_else(
        || {
            if allowance.within.is_empty() {
                "the repository".into()
            } else {
                format!("within {:?}", allowance.within)
            }
        },
        |path| format!("path {path:?}"),
    )
}

#[cfg(test)]
#[path = "item_macros_test.rs"]
mod item_macros_test;
