//! Frozen path classification and UTF-8 reads remain trusted qualification helpers.

use std::{fs, path::Path};

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

pub(crate) fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn is_test(root: &Path, path: &Path) -> bool {
    let relative = display_path(root, path);
    relative.starts_with("tests/") || relative.ends_with("_test.rs")
}
