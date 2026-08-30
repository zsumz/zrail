//! Package-manifest macro authority validation.

use super::{LockError, LockFile, valid_digest, valid_root};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    canonicalize_implementations(lock)?;
    canonicalize_sources(lock)
}

fn canonicalize_implementations(lock: &mut LockFile) -> Result<(), LockError> {
    for implementation in &lock.macro_implementations {
        if implementation.package.trim().is_empty() || !valid_root(&implementation.directory) {
            return Err(LockError(format!(
                "locked macro implementation has invalid package identity: {} in {}",
                implementation.package, implementation.directory
            )));
        }
        if !valid_digest(&implementation.inputs_sha256) {
            return Err(LockError(format!(
                "locked macro implementation {} has invalid inputs_sha256",
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

fn canonicalize_sources(lock: &mut LockFile) -> Result<(), LockError> {
    for source in &lock.macro_sources {
        if source.allowance.trim().is_empty()
            || source.package.trim().is_empty()
            || source.version.trim().is_empty()
            || source.source.trim().is_empty()
        {
            return Err(LockError::new(
                "locked macro source requires complete allowance and package identity",
            ));
        }
        if source.source.starts_with("registry+")
            && !source.checksum.as_deref().is_some_and(valid_digest)
        {
            return Err(LockError::new(format!(
                "locked registry macro source {} requires a SHA-256 checksum",
                source.allowance
            )));
        }
        if !source.source.starts_with("registry+") && source.checksum.is_some() {
            return Err(LockError::new(format!(
                "locked non-registry macro source {} may not carry a checksum",
                source.allowance
            )));
        }
    }
    lock.macro_sources.sort();
    super::ensure_unique(
        lock.macro_sources
            .iter()
            .map(|source| source.allowance.as_str()),
        "locked macro source allowance",
    )
}
