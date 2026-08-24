//! Exact item-macro manifest identity is normalized and unique.

use super::{ensure_unique, valid_digest, valid_root};
use crate::{LockError, LockFile};

pub(super) fn canonicalize(lock: &mut LockFile) -> Result<(), LockError> {
    for manifest in &lock.item_macro_manifests {
        if manifest.name.trim().is_empty() {
            return Err(LockError::new(
                "locked item-macro manifest name may not be empty",
            ));
        }
        for (label, path) in [
            ("invocation", &manifest.invocation_path),
            ("manifest", &manifest.manifest_path),
        ] {
            if path == "." || !valid_root(path) {
                return Err(LockError::new(format!(
                    "locked item-macro {label} path is invalid: {path}"
                )));
            }
        }
        if !valid_digest(&manifest.manifest_sha256) || !valid_digest(&manifest.invocation_sha256) {
            return Err(LockError::new(format!(
                "locked item-macro manifest {} has an invalid digest",
                manifest.manifest_path
            )));
        }
    }
    lock.item_macro_manifests.sort();
    ensure_unique(
        lock.item_macro_manifests
            .iter()
            .map(|manifest| format!("{}:{}", manifest.name, manifest.invocation_path)),
        "locked item-macro manifest",
    )
}
