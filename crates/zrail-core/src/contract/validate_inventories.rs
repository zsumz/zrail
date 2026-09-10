//! Exact inventory selections and quantities must be bounded and unambiguous.

use std::collections::BTreeSet;

use super::{
    Contract, RustInventoryAssertion, RustInventoryRule, RustInventorySubject,
    validate_limits::ValidationErrors,
    validate_paths::{validate_repository_literal, validate_repository_pattern},
    validate_sets::require_reason,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let rules = &contract.source.rust.inventories;
    if rules.len() > 1_024 {
        errors.push("source.rust.inventories exceeds the 1024-rule safety limit".into());
    }
    let mut names = BTreeSet::new();
    for rule in rules {
        if rule.name.is_empty()
            || rule.name.len() > 128
            || rule.name.trim() != rule.name
            || !names.insert(&rule.name)
        {
            errors
                .push("Rust inventories require unique nonempty names of at most 128 bytes".into());
        }
        require_reason("Rust inventory", &rule.name, &rule.reason, errors);
        if rule.include.is_empty() || rule.include.len() + rule.exclude.len() > 64 {
            errors.push("Rust inventories require 1..64 selectors".into());
        }
        for patterns in [&rule.include, &rule.exclude] {
            unique(patterns, "Rust inventory selectors", errors);
            for pattern in patterns {
                validate_repository_pattern(pattern, errors);
                canonical(pattern, errors);
            }
        }
        let names = rule.subject.names();
        if names.is_empty() || names.len() > 128 {
            errors.push("Rust inventories require 1..128 subject selectors".into());
        }
        unique(names, "Rust inventory subject selectors", errors);
        for name in names {
            let valid = match rule.subject {
                RustInventorySubject::WrittenMethods { .. }
                | RustInventorySubject::WrittenPathsContaining { .. }
                | RustInventorySubject::WrittenImportRenames { .. }
                | RustInventorySubject::WrittenFileModules { .. }
                | RustInventorySubject::WrittenTraitImpls { .. } => identifier(name),
                RustInventorySubject::WrittenExpressionPaths { .. }
                | RustInventorySubject::WrittenFileEnumVariants { .. } => {
                    name.split("::").count() == 2 && name.split("::").all(identifier)
                }
            };
            if !valid {
                errors.push(format!("unsupported written inventory subject {name:?}"));
            }
        }
        if let RustInventorySubject::WrittenTraitImpls {
            implementing_types, ..
        } = &rule.subject
        {
            if implementing_types.values().map(Vec::len).sum::<usize>() > 128 {
                errors.push("trait impl inventories exceed 128 implementing-type selectors".into());
            }
            for (name, types) in implementing_types {
                if !names.contains(name)
                    || types.is_empty()
                    || !types.iter().all(|name| identifier(name))
                {
                    errors.push("trait impl type filters require selected traits and nonempty exact identifiers".into());
                }
                unique(types, "trait impl type selectors", errors);
            }
        }
        match &rule.assertion {
            RustInventoryAssertion::Count { minimum, maximum } => {
                if *minimum > 50_000
                    || maximum.is_some_and(|value| value > 50_000)
                    || maximum.is_some_and(|value| value < *minimum)
                    || (*minimum == 0 && maximum.is_none())
                {
                    errors.push(
                        "Rust inventory count must be nonvacuous, ordered, and at most 50000"
                            .into(),
                    );
                }
            }
            RustInventoryAssertion::ExactCounts { counts } => {
                validate_pairs(
                    rule,
                    counts
                        .iter()
                        .map(|count| (count.path.as_str(), count.name.as_str(), count.count)),
                    errors,
                );
            }
            RustInventoryAssertion::ExactOwners { owners } => {
                validate_pairs(
                    rule,
                    owners
                        .iter()
                        .map(|owner| (owner.path.as_str(), owner.name.as_str(), 1)),
                    errors,
                );
            }
        }
    }
}

fn validate_pairs<'a>(
    rule: &RustInventoryRule,
    entries: impl Iterator<Item = (&'a str, &'a str, usize)>,
    errors: &mut ValidationErrors,
) {
    let mut pairs = BTreeSet::new();
    let mut total = 0_usize;
    let mut length = 0;
    for (path, name, count) in entries {
        length += 1;
        validate_repository_literal(path, errors);
        canonical(path, errors);
        if std::path::Path::new(path).extension() != Some(std::ffi::OsStr::new("rs"))
            || !selected_identity(&rule.subject, name)
            || !rule
                .include
                .iter()
                .any(|pattern| crate::glob_matches(pattern, path))
            || rule
                .exclude
                .iter()
                .any(|pattern| crate::glob_matches(pattern, path))
        {
            errors.push(format!(
                "Rust inventory pair {:?} falls outside its subject or file selection",
                (path, name)
            ));
        }
        if !pairs.insert((path, name)) {
            errors.push("Rust inventory contains duplicate file/subject pairs".into());
        }
        total = total.saturating_add(count);
        if count == 0 {
            errors.push(
                "exact Rust inventory entries require positive counts; omit zero pairs".into(),
            );
        }
    }
    if length > 4_096 || total > 50_000 {
        errors.push("exact Rust inventory exceeds 4096 pairs or 50000 occurrences".into());
    }
}

fn selected_identity(subject: &RustInventorySubject, identity: &str) -> bool {
    let name = if matches!(subject, RustInventorySubject::WrittenImportRenames { .. }) {
        let Some((source, alias)) = identity.split_once(" as ") else {
            return false;
        };
        if !(alias == "_" || identifier(alias)) {
            return false;
        }
        source
    } else if let RustInventorySubject::WrittenTraitImpls {
        implementing_types, ..
    } = subject
    {
        let Some((trait_name, type_name)) = identity.split_once(" for ") else {
            return false;
        };
        if !identifier(type_name)
            || implementing_types
                .get(trait_name)
                .is_some_and(|types| !types.iter().any(|name| name == type_name))
        {
            return false;
        }
        trait_name
    } else {
        identity
    };
    subject.names().iter().any(|selected| selected == name)
}

fn canonical(path: &str, errors: &mut ValidationErrors) {
    if path.contains('\0')
        || path.split('/').any(str::is_empty)
        || crate::normalize_relative(std::path::Path::new(path)).is_ok_and(|p| p != path)
    {
        errors.push(format!("Rust inventory path is not canonical: {path:?}"));
    }
}

fn unique(values: &[String], what: &str, errors: &mut ValidationErrors) {
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        errors.push(format!("{what} contains duplicate values"));
    }
}

fn identifier(name: &str) -> bool {
    let name = name.strip_prefix("r#").unwrap_or(name);
    let mut chars = name.chars();
    name.len() <= 128
        && name != "_"
        && chars
            .next()
            .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

pub(super) fn item_count(contract: &Contract) -> usize {
    contract
        .source
        .rust
        .inventories
        .iter()
        .map(|rule| {
            1 + rule.include.len()
                + rule.exclude.len()
                + rule.subject.names().len()
                + match &rule.subject {
                    RustInventorySubject::WrittenTraitImpls {
                        implementing_types, ..
                    } => implementing_types
                        .values()
                        .map(|types| 1 + types.len())
                        .sum::<usize>(),
                    _ => 0,
                }
                + match &rule.assertion {
                    RustInventoryAssertion::Count { .. } => 1,
                    RustInventoryAssertion::ExactCounts { counts } => counts.len(),
                    RustInventoryAssertion::ExactOwners { owners } => owners.len(),
                }
        })
        .sum()
}

#[cfg(test)]
#[path = "validate_inventories_test.rs"]
mod validate_inventories_test;
