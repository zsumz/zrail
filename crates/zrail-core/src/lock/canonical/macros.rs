//! Package-manifest macro authority validation.

use super::{LockError, LockFile, valid_digest, valid_root};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    canonicalize_implementations(lock)
}

fn canonicalize_implementations(lock: &mut LockFile) -> Result<(), LockError> {
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
