//! Declared fixture owner for filesystem access.

use std::fs::File;
use std::path::Path;

pub(crate) fn accept(file: File) -> File { file }

pub(crate) fn metadata_exists(path: &Path) -> bool { std::fs::metadata(path).is_ok() }
