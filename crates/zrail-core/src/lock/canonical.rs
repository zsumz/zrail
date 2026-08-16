//! Validation and deterministic ordering for resolved lock state.

mod dependency;
mod macros;

use std::{fmt, path::Path};

use super::{LockError, LockFile};

impl LockFile {
    pub fn canonicalize(&mut self) -> Result<(), LockError> {
        validate_header(self)?;
        canonicalize_packages(self)?;
        canonicalize_generated(self)?;
        canonicalize_gates(self)?;
        macros::canonicalize(self)?;
        canonicalize_ratchets(self)?;
        Ok(())
    }
}

fn validate_header(lock: &LockFile) -> Result<(), LockError> {
    if lock.schema == 0 {
        return Err(LockError("zrail.lock schema must be positive".into()));
    }
    if lock.semantics == 0 {
        return Err(LockError("zrail.lock semantics must be positive".into()));
    }
    if lock.producer.trim().is_empty() {
        return Err(LockError("zrail.lock producer may not be empty".into()));
    }
    super::compatibility::validate_epochs(lock.schema, lock.semantics)?;
    if !valid_digest(&lock.contract_sha256) {
        return Err(LockError(
            "zrail.lock contract_sha256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    Ok(())
}

fn canonicalize_packages(lock: &mut LockFile) -> Result<(), LockError> {
    let semantics = lock.semantics;
    for package in &mut lock.packages {
        if package.name.trim().is_empty() {
            return Err(LockError("locked package names may not be empty".into()));
        }
        for dependency in &mut package.dependencies {
            if dependency.name.trim().is_empty() {
                return Err(LockError(format!(
                    "dependency names in package {} may not be empty",
                    package.name
                )));
            }
            dependency::canonicalize(dependency, semantics)?;
        }
        package.dependencies.sort();
        if package
            .dependencies
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(LockError(format!(
                "duplicate dependency in package {}",
                package.name
            )));
        }
    }
    lock.packages
        .sort_by(|left, right| left.name.cmp(&right.name));
    ensure_unique(
        lock.packages.iter().map(|package| package.name.as_str()),
        "locked package",
    )
}

fn canonicalize_generated(lock: &mut LockFile) -> Result<(), LockError> {
    for generated in &lock.generated {
        if !valid_root(&generated.root) {
            return Err(LockError(format!(
                "locked generated root is not a normalized repository path: {}",
                generated.root
            )));
        }
        if !valid_digest(&generated.manifest_sha256) {
            return Err(LockError(format!(
                "locked generated root {} has an invalid manifest_sha256",
                generated.root
            )));
        }
    }
    lock.generated
        .sort_by(|left, right| left.root.cmp(&right.root));
    ensure_unique(
        lock.generated
            .iter()
            .map(|generated| generated.root.as_str()),
        "locked generated root",
    )
}

fn canonicalize_gates(lock: &mut LockFile) -> Result<(), LockError> {
    let semantics = lock.semantics;
    for gate in &mut lock.gates {
        if !valid_name(&gate.name) {
            return Err(LockError(format!(
                "locked gate name is invalid: {}",
                gate.name
            )));
        }
        if gate.path == "." || gate.path == "zrail.lock" || !valid_root(&gate.path) {
            return Err(LockError(format!(
                "locked gate path is not a normalized repository file: {}",
                gate.path
            )));
        }
        if !valid_digest(&gate.sha256) {
            return Err(LockError(format!(
                "locked gate {} has an invalid sha256",
                gate.name
            )));
        }
        if semantics < 7 && !gate.inputs.is_empty() {
            return Err(LockError(format!(
                "locked gate {} inputs require semantic epoch 7",
                gate.name
            )));
        }
        for input in &gate.inputs {
            if input.path == "." || input.path == "zrail.lock" || !valid_root(&input.path) {
                return Err(LockError(format!(
                    "locked gate {} input is not a normalized repository file: {}",
                    gate.name, input.path
                )));
            }
            if input.path == gate.path {
                return Err(LockError(format!(
                    "locked gate {} repeats its primary path as an input",
                    gate.name
                )));
            }
            if !valid_digest(&input.sha256) {
                return Err(LockError(format!(
                    "locked gate {} input {} has an invalid sha256",
                    gate.name, input.path
                )));
            }
        }
        gate.inputs.sort();
        ensure_unique(
            gate.inputs.iter().map(|input| input.path.as_str()),
            &format!("locked gate {} input", gate.name),
        )?;
    }
    lock.gates.sort_by(|left, right| left.name.cmp(&right.name));
    ensure_unique(
        lock.gates.iter().map(|gate| gate.name.as_str()),
        "locked gate",
    )?;
    ensure_unique(
        lock.gates.iter().map(|gate| gate.path.as_str()),
        "locked gate path",
    )
}

fn canonicalize_ratchets(lock: &mut LockFile) -> Result<(), LockError> {
    for ratchet in &lock.ratchets {
        if ratchet.rule.trim().is_empty() || ratchet.target.trim().is_empty() {
            return Err(LockError(
                "locked ratchets require non-empty rule and target".into(),
            ));
        }
        if ratchet.value == 0 {
            return Err(LockError(format!(
                "locked ratchet {}:{} must be positive",
                ratchet.rule, ratchet.target
            )));
        }
    }
    lock.ratchets
        .sort_by(|left, right| (&left.rule, &left.target).cmp(&(&right.rule, &right.target)));
    ensure_unique(
        lock.ratchets
            .iter()
            .map(|ratchet| format!("{}:{}", ratchet.rule, ratchet.target)),
        "locked ratchet",
    )
}

pub(super) fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub(super) fn valid_root(root: &str) -> bool {
    if root == "." {
        return true;
    }
    !root.contains(['*', '?'])
        && crate::path::normalize_relative(Path::new(root))
            .is_ok_and(|normalized| !normalized.is_empty() && normalized == root)
}

fn ensure_unique<T>(values: impl Iterator<Item = T>, label: &str) -> Result<(), LockError>
where
    T: fmt::Display,
{
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if !seen.insert(value.to_string()) {
            return Err(LockError(format!("duplicate {label} {value}")));
        }
    }
    Ok(())
}
