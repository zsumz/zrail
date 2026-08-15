//! Deliberate glob-import escape from an exact call owner.

use std::fs::*;
use std::path::Path;

pub(crate) fn metadata_exists(path: &Path) -> bool { metadata(path).is_ok() }
