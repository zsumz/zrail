//! Exact inventory selections and quantities must be bounded and unambiguous.

use std::collections::BTreeSet;

use super::{
    Contract, RustInventoryAssertion, RustInventorySubject,
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
        let RustInventorySubject::WrittenMethods { names } = &rule.subject;
        if names.is_empty() || names.len() > 128 {
            errors.push("written-method inventories require 1..128 identifiers".into());
        }
        unique(names, "written-method identifiers", errors);
        for name in names {
            if !identifier(name) {
                errors.push(format!("unsupported written method identifier {name:?}"));
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
                let mut pairs = BTreeSet::new();
                let mut total = 0_usize;
                for count in counts {
                    validate_repository_literal(&count.path, errors);
                    canonical(&count.path, errors);
                    if std::path::Path::new(&count.path).extension()
                        != Some(std::ffi::OsStr::new("rs"))
                        || !names.contains(&count.name)
                        || !rule
                            .include
                            .iter()
                            .any(|p| crate::glob_matches(p, &count.path))
                        || rule
                            .exclude
                            .iter()
                            .any(|p| crate::glob_matches(p, &count.path))
                    {
                        errors.push(format!(
                            "Rust inventory pair {:?} falls outside its subject or file selection",
                            (&count.path, &count.name)
                        ));
                    }
                    if !pairs.insert((&count.path, &count.name)) {
                        errors.push("Rust inventory contains duplicate file/subject pairs".into());
                    }
                    total = total.saturating_add(count.count);
                    if count.count == 0 {
                        errors.push(
                            "exact Rust inventory entries require positive counts; omit zero pairs"
                                .into(),
                        );
                    }
                }
                if counts.len() > 4_096 || total > 50_000 {
                    errors.push(
                        "exact Rust inventory exceeds 4096 pairs or 50000 occurrences".into(),
                    );
                }
            }
        }
    }
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
            let RustInventorySubject::WrittenMethods { names } = &rule.subject;
            1 + rule.include.len()
                + rule.exclude.len()
                + names.len()
                + match &rule.assertion {
                    RustInventoryAssertion::Count { .. } => 1,
                    RustInventoryAssertion::ExactCounts { counts } => counts.len(),
                }
        })
        .sum()
}

#[cfg(test)]
#[path = "validate_inventories_test.rs"]
mod validate_inventories_test;
