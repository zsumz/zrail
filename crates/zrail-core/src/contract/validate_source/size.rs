//! Scoped budgets reject ambiguous declarations and unbounded exception authority.

use crate::SizeExceptionMetadata;
use std::collections::BTreeSet;

use super::super::{
    Contract, ScopedBudgetContract, SizeExceptionContract,
    validate_limits::ValidationErrors,
    validate_paths::{
        validate_package_name, validate_repository_literal, validate_repository_pattern,
    },
    validate_sets::require_reason,
};

const MAX_OVERRIDES: usize = 256;
const MAX_SELECTORS: usize = 64;

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let Some(policy) = &contract.source.rust.budgets else {
        return;
    };
    if policy.overrides.len() > MAX_OVERRIDES {
        errors.push(format!(
            "scoped budgets exceed the {MAX_OVERRIDES}-override safety limit"
        ));
        return;
    }
    let mut names = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    for scope in &policy.overrides {
        if scope.name.trim().is_empty() || !names.insert(&scope.name) {
            errors.push(format!(
                "scoped budget requires a unique nonempty name: {:?}",
                scope.name
            ));
        }
        require_reason("scoped budget", &scope.name, &scope.reason, errors);
        validate_scope(scope, errors);
        let selector = (
            scope.include.iter().collect::<BTreeSet<_>>(),
            scope.exclude.iter().collect::<BTreeSet<_>>(),
            scope.packages.iter().collect::<BTreeSet<_>>(),
            scope.roles.iter().collect::<BTreeSet<_>>(),
        );
        if !selectors.insert(selector) {
            errors.push(format!(
                "scoped budget {:?} duplicates a same-tier selector",
                scope.name
            ));
        }
    }
    let mut paths = BTreeSet::new();
    for exception in &policy.exceptions {
        if !paths.insert(&exception.path) {
            errors.push(format!("duplicate size exception for {:?}", exception.path));
        }
        validate_exception(exception, policy.exception_metadata, errors);
    }
}

fn validate_scope(scope: &ScopedBudgetContract, errors: &mut ValidationErrors) {
    if scope.include.is_empty() {
        errors.push(format!(
            "scoped budget {:?} requires an include selector",
            scope.name
        ));
    }
    let count =
        scope.include.len() + scope.exclude.len() + scope.packages.len() + scope.roles.len();
    if count > MAX_SELECTORS {
        errors.push(format!(
            "scoped budget {:?} exceeds the {MAX_SELECTORS}-selector safety limit",
            scope.name
        ));
    }
    for patterns in [&scope.include, &scope.exclude] {
        unique(patterns, &scope.name, errors);
        for pattern in patterns {
            validate_repository_pattern(pattern, errors);
        }
    }
    unique(&scope.packages, &scope.name, errors);
    unique(&scope.roles, &scope.name, errors);
    for package in &scope.packages {
        validate_package_name(package, errors);
    }
    let budget = scope.budget;
    if budget.target == 0 || budget.hard < budget.target {
        errors.push(format!(
            "scoped budget {:?} requires 0 < target <= hard",
            scope.name
        ));
    }
    if budget
        .soft
        .is_some_and(|soft| soft < budget.target || soft > budget.hard)
    {
        errors.push(format!(
            "scoped budget {:?} requires target <= soft <= hard",
            scope.name
        ));
    }
}

fn validate_exception(
    exception: &SizeExceptionContract,
    metadata: crate::SizeExceptionMetadata,
    errors: &mut ValidationErrors,
) {
    validate_repository_literal(&exception.path, errors);
    if !std::path::Path::new(&exception.path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs") || extension == "rsi")
    {
        errors.push(format!(
            "size exception must name exact Rust source: {:?}",
            exception.path
        ));
    }
    require_reason("size exception", &exception.path, &exception.reason, errors);
    if exception.hard == 0 {
        errors.push(format!(
            "size exception {:?} requires a positive hard bound",
            exception.path
        ));
    }
    for (field, value) in [
        ("owner", &exception.owner),
        ("issue", &exception.issue),
        ("tracking", &exception.tracking),
    ] {
        if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
            errors.push(format!(
                "size exception {:?} has empty {field}",
                exception.path
            ));
        }
    }
    if exception.owner.is_some() != exception.issue.is_some()
        || exception.tracking.is_none() && exception.owner.is_none()
    {
        errors.push(format!(
            "size exception {:?} requires an owner/issue pair or tracking identity",
            exception.path
        ));
    }
    let valid = match metadata {
        SizeExceptionMetadata::OwnerIssueOrTracking => true,
        SizeExceptionMetadata::OwnerIssue => exception.owner.is_some() && exception.issue.is_some(),
        SizeExceptionMetadata::Tracking => exception.tracking.is_some(),
        SizeExceptionMetadata::OwnerIssueAndTracking => {
            exception.owner.is_some() && exception.issue.is_some() && exception.tracking.is_some()
        }
    };
    if !valid {
        errors.push(format!(
            "size exception {:?} lacks required {metadata:?} metadata",
            exception.path
        ));
    }
}

fn unique<T: Ord>(values: &[T], name: &str, errors: &mut ValidationErrors) {
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        errors.push(format!(
            "scoped budget {name:?} contains a duplicate selector"
        ));
    }
}

#[cfg(test)]
#[path = "size_test.rs"]
mod size_test;
