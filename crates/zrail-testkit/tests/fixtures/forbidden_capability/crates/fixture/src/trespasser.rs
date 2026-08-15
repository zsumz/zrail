//! Deliberate use of an owned capability outside its owner.

use std::fs as hidden_fs;

pub(crate) fn exists(path: &std::path::Path) -> bool { hidden_fs::metadata(path).is_ok() }
