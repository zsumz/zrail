//! Cross-section validation for merged architecture contracts.

mod owner;

use std::collections::BTreeSet;

use super::validate_paths::{
    validate_package_name, validate_package_pattern, validate_repository_literal,
    validate_repository_pattern,
};
use super::{
    Contract, ContractError,
    validate_limits::{ValidationErrors, enforce_contract_size},
    validate_sets::{collect_unique, require_reason},
};

pub(super) fn validate_contract(contract: &Contract) -> Result<(), ContractError> {
    enforce_contract_size(contract)?;
    let mut errors = ValidationErrors::new();
    if contract.schema != 1 {
        errors.push(format!(
            "unsupported zrail contract schema {}",
            contract.schema
        ));
    }
    validate_adapters(contract, &mut errors);
    validate_repository(contract, &mut errors);
    validate_budgets(contract, &mut errors);
    super::validate_source::validate_source_contract(contract, &mut errors);
    validate_layers(contract, &mut errors);
    validate_named_rules(contract, &mut errors);
    owner::validate_contract(contract, &mut errors);
    super::validate_ratchet::validate(contract, &mut errors);
    super::validate_evidence::validate(contract, &mut errors);
    super::validate_sets::validate_sets(contract, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ContractError::many(errors.finish()))
    }
}

fn validate_adapters(contract: &Contract, errors: &mut ValidationErrors) {
    let mut names = BTreeSet::new();
    for adapter in &contract.adapters {
        if adapter.trim().is_empty() {
            errors.push("adapter names may not be empty".into());
        } else if !names.insert(adapter.as_str()) {
            errors.push(format!("duplicate adapter {adapter:?}"));
        }
    }
    if !names.contains("rust") {
        errors.push("the initial zrail engine requires the rust adapter".into());
    }
}

fn validate_repository(contract: &Contract, errors: &mut ValidationErrors) {
    if contract.repository.roots.is_empty() {
        errors.push("repository.roots must name at least one source root".into());
    }
    for path in &contract.repository.roots {
        validate_repository_literal(path, errors);
    }
    for path in &contract.repository.exclude {
        validate_repository_pattern(path, errors);
    }
}

fn validate_budgets(contract: &Contract, errors: &mut ValidationErrors) {
    let Some(size) = &contract.source.rust.size else {
        return;
    };
    for (name, budget) in [
        ("facade", size.facade),
        ("implementation", size.implementation),
        ("test", size.test),
        ("auxiliary", size.auxiliary),
    ] {
        if budget.target == 0 || budget.hard == 0 {
            errors.push(format!("source.rust.size.{name} must be positive"));
        }
        if budget.target > budget.hard {
            errors.push(format!(
                "source.rust.size.{name}.target exceeds its hard ceiling"
            ));
        }
    }
}

fn validate_layers(contract: &Contract, errors: &mut ValidationErrors) {
    let names = collect_unique(
        contract.layers.iter().map(|layer| layer.name.as_str()),
        "layer",
        errors,
    );
    let mut assigned_packages = BTreeSet::new();
    let mut used_profiles = BTreeSet::new();
    for layer in &contract.layers {
        require_reason("layer", &layer.name, &layer.reason, errors);
        if layer.packages.is_empty() {
            errors.push(format!("layer {:?} assigns no packages", layer.name));
        }
        for package in &layer.packages {
            validate_package_pattern(package, errors);
            if !assigned_packages.insert(package) {
                errors.push(format!(
                    "package {package:?} is assigned to multiple layers"
                ));
            }
        }
        for dependency in &layer.may_depend_on {
            if dependency == &layer.name {
                errors.push(format!("layer {:?} may not depend on itself", layer.name));
            } else if !names.contains(dependency.as_str()) {
                errors.push(format!(
                    "layer {:?} references missing layer {dependency:?}",
                    layer.name
                ));
            }
        }
        for profile in &layer.profiles {
            used_profiles.insert(profile.as_str());
            if !contract.profiles.contains_key(profile) {
                errors.push(format!(
                    "layer {:?} references missing profile {profile:?}",
                    layer.name
                ));
            }
        }
    }
    for profile in contract.profiles.keys() {
        if !used_profiles.contains(profile.as_str()) {
            errors.push(format!(
                "profile {profile:?} is defined but assigned to no layer"
            ));
        }
    }
}

fn validate_named_rules(contract: &Contract, errors: &mut ValidationErrors) {
    let mut names = BTreeSet::new();
    for (kind, name, reason) in contract
        .dependency_rules
        .iter()
        .map(|rule| ("dependency", rule.name.as_str(), rule.reason.as_str()))
        .chain(
            contract
                .scopes
                .iter()
                .map(|scope| ("scope", scope.name.as_str(), scope.reason.as_str())),
        )
        .chain(
            contract
                .owners
                .iter()
                .map(|owner| ("owner", owner.name.as_str(), owner.reason.as_str())),
        )
    {
        if name.trim().is_empty() {
            errors.push(format!("{kind} names may not be empty"));
        } else if !names.insert(name) {
            errors.push(format!("duplicate rule name {name:?}"));
        }
        require_reason(kind, name, reason, errors);
    }
    for rule in &contract.dependency_rules {
        if rule.from.trim().is_empty() || rule.deny.is_empty() {
            errors.push(format!(
                "dependency rule {:?} requires from and deny",
                rule.name
            ));
        }
        validate_package_name(&rule.from, errors);
        for package in &rule.deny {
            validate_package_name(package, errors);
        }
    }
    for scope in &contract.scopes {
        if scope.include.is_empty() {
            errors.push(format!("scope {:?} includes no source", scope.name));
        }
        if scope.symbols.deny.is_empty() {
            errors.push(format!("scope {:?} denies no symbols", scope.name));
        }
        for pattern in scope.include.iter().chain(&scope.exclude) {
            validate_repository_pattern(pattern, errors);
        }
    }
}

#[cfg(test)]
#[path = "validate_test.rs"]
mod validate_test;
