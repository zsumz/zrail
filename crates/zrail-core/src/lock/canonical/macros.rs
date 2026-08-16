//! Legacy definition and current package-manifest macro authority validation.

use super::{LockError, LockFile, valid_digest, valid_root};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    canonicalize_definitions(lock)?;
    canonicalize_implementations(lock)
}

fn canonicalize_implementations(lock: &mut LockFile) -> Result<(), LockError> {
    if lock.semantics < 6 && !lock.macro_implementations.is_empty() {
        return Err(LockError(
            "locked macro implementations require lock semantics 6 or newer".into(),
        ));
    }
    for implementation in &lock.macro_implementations {
        if implementation.package.trim().is_empty() || !valid_root(&implementation.directory) {
            return Err(LockError(format!(
                "locked macro implementation has invalid package identity: {} in {}",
                implementation.package, implementation.directory
            )));
        }
        if !valid_digest(&implementation.manifest_sha256) {
            return Err(LockError(format!(
                "locked macro implementation {} has invalid manifest_sha256",
                implementation.package
            )));
        }
    }
    lock.macro_implementations.sort();
    if lock.macro_implementations.windows(2).any(|pair| {
        (&pair[0].package, &pair[0].directory) == (&pair[1].package, &pair[1].directory)
    }) {
        return Err(LockError("duplicate locked macro implementation".into()));
    }
    Ok(())
}

fn canonicalize_definitions(lock: &mut LockFile) -> Result<(), LockError> {
    if lock.semantics < 5 && !lock.macros.is_empty() {
        return Err(LockError(
            "locked macro definitions require lock semantics 5 or newer".into(),
        ));
    }
    if lock.semantics >= 6 && !lock.macros.is_empty() {
        return Err(LockError(
            "locked macro definitions are legacy semantics 5 state; regenerate package implementation authority".into(),
        ));
    }
    for definition in &lock.macros {
        if !valid_root(&definition.path) || !valid_macro_name(&definition.name) {
            return Err(LockError(format!(
                "locked macro definition is invalid: {} in {}",
                definition.name, definition.path
            )));
        }
        if definition.ordinal == 0 || !valid_digest(&definition.sha256) {
            return Err(LockError(format!(
                "locked macro definition {} in {} has invalid observation state",
                definition.name, definition.path
            )));
        }
    }
    lock.macros.sort();
    if lock.macros.windows(2).any(|pair| {
        (&pair[0].path, &pair[0].name, pair[0].ordinal)
            == (&pair[1].path, &pair[1].name, pair[1].ordinal)
    }) {
        return Err(LockError("duplicate locked macro definition".into()));
    }
    Ok(())
}

fn valid_macro_name(value: &str) -> bool {
    value.split("::").all(|segment| {
        let mut bytes = segment.bytes();
        bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}
