//! Lock expansion for complete generated provenance manifests.

use std::path::Path;

use zrail_core::{
    GeneratedSourceContract, LockedGeneratedSource, MAX_INPUT_BYTES, read_bytes_with_limit,
    sha256_hex,
};

pub(crate) fn locked_sources(
    repository: &Path,
    generated: &[GeneratedSourceContract],
) -> Vec<LockedGeneratedSource> {
    generated
        .iter()
        .filter_map(|contract| {
            let bytes =
                read_bytes_with_limit(&repository.join(&contract.manifest), MAX_INPUT_BYTES)
                    .ok()?;
            Some(LockedGeneratedSource {
                root: contract.root.clone(),
                manifest_sha256: sha256_hex(&bytes),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "lock_state_test.rs"]
mod lock_state_test;
