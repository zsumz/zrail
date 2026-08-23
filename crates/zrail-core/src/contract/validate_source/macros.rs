//! Macro authority is reasoned, non-overlapping, and optionally definition-narrowed.

use std::collections::BTreeSet;

use crate::contract::{
    Contract, CrateRootSource, MacroBindingMode, MacroExpansionBindings, MacroExpansionMode,
    validate_dependencies, validate_limits::ValidationErrors,
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
        if allowed.bindings == MacroExpansionBindings::None {
            if allowed.binding != MacroBindingMode::Exact {
                errors.push(format!(
                    "macro expansion allowance {:?} with bindings = \"none\" requires exact binding",
                    allowed.name
                ));
            }
            if allowed.source.is_none() && allowed.definition.is_none() {
                errors.push(format!(
                    "macro expansion allowance {:?} with bindings = \"none\" requires source or definition provenance",
                    allowed.name
                ));
            }
            if allowed
                .source
                .as_ref()
                .is_some_and(|source| !immutable_source(source))
            {
                errors.push(format!(
                    "macro expansion allowance {:?} with bindings = \"none\" requires immutable source provenance: registry sources must use an exact =major.minor.patch version and Git sources must use a full 40- or 64-hex rev",
                    allowed.name
                ));
            }
        }
    }
}

fn immutable_source(source: &CrateRootSource) -> bool {
    match source {
        CrateRootSource::Legacy => false,
        CrateRootSource::Registry { requirement, .. } => exact_registry_pin(requirement),
        CrateRootSource::Git {
            branch, tag, rev, ..
        } => branch.is_none() && tag.is_none() && rev.as_deref().is_some_and(exact_git_revision),
    }
}

fn exact_git_revision(revision: &str) -> bool {
    matches!(revision.len(), 40 | 64) && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn exact_registry_pin(requirement: &str) -> bool {
    requirement
        .strip_prefix('=')
        .is_some_and(valid_semver_version)
}

fn valid_semver_version(version: &str) -> bool {
    if version.is_empty() || version.trim() != version {
        return false;
    }
    let mut build_parts = version.split('+');
    let Some(base) = build_parts.next() else {
        return false;
    };
    if build_parts
        .next()
        .is_some_and(|build| !valid_identifiers(build, false))
        || build_parts.next().is_some()
    {
        return false;
    }
    let (core, prerelease) = base
        .split_once('-')
        .map_or((base, None), |(core, prerelease)| (core, Some(prerelease)));
    if prerelease.is_some_and(|prerelease| !valid_identifiers(prerelease, true)) {
        return false;
    }
    let mut numbers = core.split('.');
    [numbers.next(), numbers.next(), numbers.next()]
        .into_iter()
        .all(|number| number.is_some_and(valid_numeric_identifier))
        && numbers.next().is_none()
}

fn valid_identifiers(value: &str, reject_numeric_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!reject_numeric_leading_zero
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || valid_numeric_identifier(identifier))
        })
}

fn valid_numeric_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

#[cfg(test)]
#[path = "macros_test.rs"]
mod macros_test;
