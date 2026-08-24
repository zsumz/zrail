//! Exact test-mirror paths and execution-context validation.

use std::{collections::BTreeSet, ffi::OsStr, path::Path};

use crate::{Contract, MAX_TEST_MIRROR_INPUTS, TestExecutionIdentity, TestMirrorContract};

use super::{ValidationErrors, gates, validate_repository_literal};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut productions = BTreeSet::new();
    let mut tests = BTreeSet::new();
    let mut receipts = BTreeSet::new();
    for mirror in &contract.source.rust.test_mirrors {
        validate_mirror_path(contract, &mirror.production, "production", "rs", errors);
        validate_mirror_path(contract, &mirror.test, "test", "rs", errors);
        validate_mirror_path(contract, &mirror.receipt, "receipt", "json", errors);
        validate_identity(mirror, errors);
        validate_inputs(contract, mirror, errors);
        validate_execution(&mirror.execution, errors);
        super::super::validate_sets::require_reason(
            "test mirror",
            &mirror.production,
            &mirror.reason,
            errors,
        );
        insert_identity(&mut productions, &mirror.production, "production", errors);
        insert_identity(&mut tests, &mirror.test, "test", errors);
        insert_identity(&mut receipts, &mirror.receipt, "receipt", errors);
    }
}

fn validate_identity(mirror: &TestMirrorContract, errors: &mut ValidationErrors) {
    if mirror.production == mirror.test {
        errors.push(format!(
            "test mirror production and test paths must differ: {:?}",
            mirror.production
        ));
    }
    if !super::super::evidence::valid_identifier(&mirror.name) {
        errors.push(format!(
            "test mirror {:?} has an invalid exact test name {:?}",
            mirror.production, mirror.name
        ));
    }
}

fn validate_inputs(
    contract: &Contract,
    mirror: &TestMirrorContract,
    errors: &mut ValidationErrors,
) {
    if mirror.inputs.len() > MAX_TEST_MIRROR_INPUTS {
        errors.push(format!(
            "test mirror {:?} exceeds {MAX_TEST_MIRROR_INPUTS} reviewed inputs",
            mirror.production
        ));
    }
    let mut inputs = BTreeSet::new();
    for input in &mirror.inputs {
        validate_repository_literal(input, errors);
        if input == "." || input == "zrail.lock" {
            errors.push(format!("invalid test mirror input {input:?}"));
        }
        if input == &mirror.production || input == &mirror.test || input == &mirror.receipt {
            errors.push(format!(
                "test mirror input {input:?} duplicates a primary mirror path"
            ));
        }
        if gates::excluded(contract, input) {
            errors.push(format!(
                "test mirror input is hidden by repository.exclude: {input:?}"
            ));
        }
        if !inputs.insert(input.as_str()) {
            errors.push(format!("test mirror repeats input path {input:?}"));
        }
    }
    for required in ["Cargo.toml", "Cargo.lock"] {
        if !inputs.contains(required) {
            errors.push(format!(
                "test mirror {:?} must bind {required:?} as a reviewed input",
                mirror.production
            ));
        }
    }
    if !mirror.inputs.windows(2).all(|pair| pair[0] < pair[1]) {
        errors.push(format!(
            "test mirror {:?} inputs must be unique and sorted",
            mirror.production
        ));
    }
}

fn validate_execution(identity: &TestExecutionIdentity, errors: &mut ValidationErrors) {
    validate_scalar("command", &identity.command, errors);
    validate_scalar("target", &identity.target, errors);
    validate_scalar("toolchain", &identity.toolchain, errors);
    if identity.package.is_empty()
        || !identity
            .package
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        errors.push(format!(
            "test mirror execution package is invalid: {:?}",
            identity.package
        ));
    }
    let valid_features = identity.features.iter().all(|feature| {
        !feature.is_empty()
            && feature
                .bytes()
                .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b',' | b'[' | b']'))
    });
    if !valid_features || !identity.features.windows(2).all(|pair| pair[0] < pair[1]) {
        errors.push("test mirror execution features must be valid, unique, and sorted".into());
    }
}

fn validate_scalar(label: &str, value: &str, errors: &mut ValidationErrors) {
    if value.is_empty()
        || value.trim() != value
        || value
            .bytes()
            .any(|byte| matches!(byte, b'\0' | b'\r' | b'\n'))
    {
        errors.push(format!(
            "test mirror execution {label} must be a non-empty normalized line"
        ));
    }
}

fn validate_mirror_path(
    contract: &Contract,
    path: &str,
    label: &str,
    extension: &str,
    errors: &mut ValidationErrors,
) {
    validate_repository_literal(path, errors);
    if path == "." {
        errors.push(format!("test mirror {label} must name a file"));
    } else if path == "zrail.lock" {
        errors.push("zrail.lock cannot serve as test-mirror evidence".into());
    } else if Path::new(path).extension() != Some(OsStr::new(extension)) {
        errors.push(format!(
            "test mirror {label} must name a .{extension} file: {path:?}"
        ));
    }
    if gates::excluded(contract, path) {
        errors.push(format!(
            "test mirror {label} is hidden by repository.exclude: {path:?}"
        ));
    }
    if label != "receipt" && !inside_roots(contract, path) {
        errors.push(format!(
            "test mirror source {path:?} must be inside repository.roots"
        ));
    }
}

fn insert_identity<'a>(
    values: &mut BTreeSet<&'a str>,
    value: &'a str,
    label: &str,
    errors: &mut ValidationErrors,
) {
    if !values.insert(value) {
        errors.push(format!("test mirror reuses {label} path {value:?}"));
    }
}

fn inside_roots(contract: &Contract, path: &str) -> bool {
    contract
        .repository
        .roots
        .iter()
        .any(|root| root == "." || path == root || path.starts_with(&format!("{root}/")))
}
