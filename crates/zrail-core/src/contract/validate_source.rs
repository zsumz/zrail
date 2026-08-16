//! Validation for Rust source-provenance and expansion boundaries.

mod macros;

use std::collections::BTreeSet;

use super::{
    Contract, validate_limits::ValidationErrors, validate_paths::validate_repository_literal,
    validate_paths::validate_repository_pattern, validate_sets::require_reason,
};

const MAX_GENERATED_INPUT_SELECTORS: usize = 64;

pub(super) fn validate_source_contract(contract: &Contract, errors: &mut ValidationErrors) {
    validate_generated(contract, errors);
    validate_out_dir(contract, errors);
    validate_item_macros(contract, errors);
    macros::validate(contract, errors);
}

fn validate_out_dir(contract: &Contract, errors: &mut ValidationErrors) {
    let mut identities = BTreeSet::new();
    for binding in &contract.source.rust.out_dir {
        validate_repository_literal(&binding.path, errors);
        validate_repository_literal(&binding.output, errors);
        validate_repository_literal(&binding.source, errors);
        require_reason("OUT_DIR source", &binding.output, &binding.reason, errors);
        if std::path::Path::new(&binding.path)
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("rs")
        {
            errors.push(format!(
                "OUT_DIR binding origin must be a .rs source: {:?}",
                binding.path
            ));
        }
        if !has_source_extension(&binding.output) || !has_source_extension(&binding.source) {
            errors.push(format!(
                "OUT_DIR binding {:?} must map Rust source to Rust source",
                binding.output
            ));
        }
        for path in [&binding.path, &binding.source] {
            if !contract
                .repository
                .roots
                .iter()
                .any(|root| contains_path(root, path))
            {
                errors.push(format!(
                    "OUT_DIR binding path {path:?} must be inside repository.roots"
                ));
            }
        }
        if !contract
            .source
            .rust
            .generated
            .iter()
            .any(|generated| contains_path(&generated.root, &binding.source))
        {
            errors.push(format!(
                "OUT_DIR source {:?} must be inside a verified generated root",
                binding.source
            ));
        }
        if !identities.insert((&binding.path, &binding.output)) {
            errors.push(format!(
                "duplicate OUT_DIR binding for {} in {}",
                binding.output, binding.path
            ));
        }
    }
}

fn validate_generated(contract: &Contract, errors: &mut ValidationErrors) {
    for generated in &contract.source.rust.generated {
        validate_repository_literal(&generated.root, errors);
        validate_repository_literal(&generated.manifest, errors);
        if generated.inputs.len() > MAX_GENERATED_INPUT_SELECTORS {
            errors.push(format!(
                "generated source {:?} exceeds the {MAX_GENERATED_INPUT_SELECTORS}-input-selector safety limit",
                generated.root
            ));
        }
        for input in &generated.inputs {
            validate_repository_pattern(input, errors);
        }
        require_reason(
            "generated source",
            &generated.root,
            &generated.reason,
            errors,
        );
        if !contract
            .repository
            .roots
            .iter()
            .any(|root| contains_path(root, &generated.root))
        {
            errors.push(format!(
                "generated source root {:?} must be inside repository.roots",
                generated.root
            ));
        }
        if generated.target == 0 || generated.hard == 0 {
            errors.push(format!(
                "generated source {:?} budgets must be positive",
                generated.root
            ));
        }
        if generated.target > generated.hard {
            errors.push(format!(
                "generated source {:?} target exceeds its hard ceiling",
                generated.root
            ));
        }
        if !contains_path(&generated.root, &generated.manifest) {
            errors.push(format!(
                "generated manifest {:?} must be inside root {:?}",
                generated.manifest, generated.root
            ));
        }
        validate_auxiliary(generated, errors);
        validate_inputs(generated, errors);
    }
    for (index, left) in contract.source.rust.generated.iter().enumerate() {
        for right in contract.source.rust.generated.iter().skip(index + 1) {
            if contains_path(&left.root, &right.root) || contains_path(&right.root, &left.root) {
                errors.push(format!(
                    "generated source roots {:?} and {:?} overlap",
                    left.root, right.root
                ));
            }
        }
    }
}

fn validate_inputs(generated: &super::GeneratedSourceContract, errors: &mut ValidationErrors) {
    let mut inputs = BTreeSet::new();
    for input in &generated.inputs {
        if !inputs.insert(input) {
            errors.push(format!(
                "generated source {:?} contains duplicate input selector {input:?}",
                generated.root
            ));
        }
    }
}

fn validate_auxiliary(generated: &super::GeneratedSourceContract, errors: &mut ValidationErrors) {
    let mut auxiliary = BTreeSet::new();
    for path in &generated.auxiliary {
        validate_repository_literal(path, errors);
        if !has_source_extension(path) {
            errors.push(format!(
                "generated auxiliary path must name .rs or .rsi source: {path:?}"
            ));
        }
        if !auxiliary.insert(path) {
            errors.push(format!(
                "generated source {:?} contains duplicate auxiliary path {path:?}",
                generated.root
            ));
        }
    }
}

fn has_source_extension(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| matches!(extension, "rs" | "rsi"))
}

fn validate_item_macros(contract: &Contract, errors: &mut ValidationErrors) {
    let mut identities = BTreeSet::new();
    for item_macro in &contract.source.rust.item_macros {
        validate_repository_literal(&item_macro.path, errors);
        require_reason("item macro", &item_macro.name, &item_macro.reason, errors);
        if !valid_rust_path(&item_macro.name) {
            errors.push(format!(
                "item macro name must be a Rust path: {:?}",
                item_macro.name
            ));
        }
        if !identities.insert((&item_macro.path, &item_macro.name)) {
            errors.push(format!(
                "duplicate item macro exemption {} in {}",
                item_macro.name, item_macro.path
            ));
        }
    }
}

pub(super) fn valid_rust_path(path: &str) -> bool {
    !path.is_empty() && path.split("::").all(valid_identifier)
}

fn valid_identifier(identifier: &str) -> bool {
    let identifier = identifier.strip_prefix("r#").unwrap_or(identifier);
    let mut bytes = identifier.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn contains_path(root: &str, path: &str) -> bool {
    root == "." || path == root || path.starts_with(&format!("{root}/"))
}
