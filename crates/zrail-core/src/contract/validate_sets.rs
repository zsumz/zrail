//! Duplicate and empty values are rejected before they become silent policy drift.

use std::collections::BTreeSet;

use super::{Contract, Effect, validate_limits::ValidationErrors};

pub(super) fn validate_sets(contract: &Contract, errors: &mut ValidationErrors) {
    strings("repository.roots", &contract.repository.roots, errors);
    strings("repository.exclude", &contract.repository.exclude, errors);
    strings(
        "source.rust.hygiene.deny_methods",
        &contract.source.rust.hygiene.deny_methods,
        errors,
    );
    strings(
        "source.rust.hygiene.deny_macros",
        &contract.source.rust.hygiene.deny_macros,
        errors,
    );
    strings(
        "source.rust.generated.root",
        &contract
            .source
            .rust
            .generated
            .iter()
            .map(|generated| generated.root.clone())
            .collect::<Vec<_>>(),
        errors,
    );
    for (name, profile) in &contract.profiles {
        if name.trim().is_empty() {
            errors.push("profile names may not be empty".into());
        }
        if profile.effects.deny.is_empty() {
            errors.push(format!("profile {name:?} denies no effects"));
        }
        effects(
            &format!("profiles.{name}.effects.deny"),
            &profile.effects.deny,
            errors,
        );
    }
    for layer in &contract.layers {
        strings(
            &format!("layer.{}.packages", layer.name),
            &layer.packages,
            errors,
        );
        strings(
            &format!("layer.{}.may_depend_on", layer.name),
            &layer.may_depend_on,
            errors,
        );
        strings(
            &format!("layer.{}.profiles", layer.name),
            &layer.profiles,
            errors,
        );
    }
    for rule in &contract.dependency_rules {
        strings(
            &format!("dependency.{}.deny", rule.name),
            &rule.deny,
            errors,
        );
    }
    for scope in &contract.scopes {
        strings(
            &format!("scope.{}.include", scope.name),
            &scope.include,
            errors,
        );
        strings(
            &format!("scope.{}.exclude", scope.name),
            &scope.exclude,
            errors,
        );
        strings(
            &format!("scope.{}.symbols.deny", scope.name),
            &scope.symbols.deny,
            errors,
        );
    }
    for owner in &contract.owners {
        strings(
            &format!("owner.{}.within", owner.name),
            &owner.within,
            errors,
        );
        strings(&format!("owner.{}.allow", owner.name), &owner.allow, errors);
    }
}

fn strings(label: &str, values: &[String], errors: &mut ValidationErrors) {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            errors.push(format!("{label} may not contain an empty value"));
        } else if !seen.insert(value) {
            errors.push(format!("{label} contains duplicate value {value:?}"));
        }
    }
}

fn effects(label: &str, values: &[Effect], errors: &mut ValidationErrors) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            errors.push(format!("{label} contains duplicate effect {value:?}"));
        }
    }
}

pub(super) fn collect_unique<'a>(
    values: impl Iterator<Item = &'a str>,
    kind: &str,
    errors: &mut ValidationErrors,
) -> BTreeSet<&'a str> {
    let mut names = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            errors.push(format!("{kind} names may not be empty"));
        } else if !names.insert(value) {
            errors.push(format!("duplicate {kind} {value:?}"));
        }
    }
    names
}

pub(super) fn require_reason(kind: &str, name: &str, reason: &str, errors: &mut ValidationErrors) {
    if reason.trim().is_empty() {
        errors.push(format!("{kind} {name:?} requires a reason"));
    }
}
