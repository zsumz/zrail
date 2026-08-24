//! Completeness certificates bind the exact authority inputs, not advisory counts.

use super::{ensure_unique, valid_digest, valid_root};
use crate::{LOCK_SEMANTICS, LockError, LockFile};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    if lock.semantics == LOCK_SEMANTICS && lock.analysis.is_none() {
        return Err(LockError::new(
            "current zrail.lock semantics require an analysis certificate",
        ));
    }
    let Some(analysis) = &mut lock.analysis else {
        return Ok(());
    };
    if !valid_digest(&analysis.inventory_sha256) || !valid_digest(&analysis.exclusions_sha256) {
        return Err(LockError::new(
            "locked analysis inventory and exclusion digests must be exact SHA-256 values",
        ));
    }
    if analysis
        .cargo_lock_sha256
        .as_deref()
        .is_some_and(|digest| !valid_digest(digest))
    {
        return Err(LockError::new(
            "locked analysis Cargo.lock digest must be an exact SHA-256 value",
        ));
    }
    if analysis.unresolved_bindings != 0 {
        return Err(LockError::new(
            "zrail.lock cannot certify unresolved analysis bindings",
        ));
    }
    if analysis.analyzer_semantics != lock.semantics {
        return Err(LockError::new(
            "locked analysis semantics must match zrail.lock semantics",
        ));
    }
    for source in &analysis.contract_sources {
        if source.path == "." || !valid_root(&source.path) {
            return Err(LockError::new(format!(
                "locked contract source is not a normalized repository file: {}",
                source.path
            )));
        }
        if !valid_digest(&source.sha256) {
            return Err(LockError::new(format!(
                "locked contract source {} has an invalid sha256",
                source.path
            )));
        }
    }
    analysis.contract_sources.sort();
    ensure_unique(
        analysis
            .contract_sources
            .iter()
            .map(|source| source.path.as_str()),
        "locked contract source",
    )
}
