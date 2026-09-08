//! The frozen final iterator expression accepts injected results, not a simulated OS directory read.

use super::physical::contains_rust_source;
use std::{fs, io};

pub(super) fn contains_from_entries(
    entries: impl Iterator<Item = io::Result<fs::DirEntry>>,
) -> bool {
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            contains_rust_source(&path)
        } else {
            path.extension().is_some_and(|extension| extension == "rs")
        }
    })
}
