//! Deliberate indirect invocation inside a declared call owner.

use std::path::Path;

pub(crate) fn metadata_exists(path: &Path) -> bool {
    let direct = std::fs::metadata(path).is_ok();
    let metadata = std::fs::metadata;
    direct && metadata(path).is_ok()
}
