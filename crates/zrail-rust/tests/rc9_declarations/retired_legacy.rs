//! Original raw-text assertion and fallible physical walker remain explicit qualification oracles.

use std::{fs, path::Path};

pub(super) fn check_construction(construction: &str) {
    assert!(
        !construction.contains("new_legacy") && !construction.contains("LegacyBackend"),
        "reactor construction must have no legacy path"
    );
}

pub(super) fn contains_rust_source(directory: &Path) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            contains_rust_source(&path)
        } else {
            path.extension().is_some_and(|extension| extension == "rs")
        }
    })
}
