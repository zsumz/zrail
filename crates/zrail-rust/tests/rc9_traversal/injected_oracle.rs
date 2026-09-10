//! Frozen failure expressions receive explicit injected Results, not claimed OS entry failures.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn directory(directory: &Path, result: io::Result<fs::ReadDir>) {
    let _entries =
        result.unwrap_or_else(|error| panic!("read directory {}: {error}", directory.display()));
}

pub(super) fn entries(entries: impl Iterator<Item = io::Result<fs::DirEntry>>) {
    let _entries = entries
        .map(|entry| entry.unwrap_or_else(|error| panic!("read directory entry: {error}")))
        .collect::<Vec<_>>();
}

pub(super) struct TypeEntry {
    pub(super) entry: fs::DirEntry,
    pub(super) kind: io::ErrorKind,
}
impl TypeEntry {
    fn path(&self) -> PathBuf {
        self.entry.path()
    }
    fn file_type(&self) -> io::Result<fs::FileType> {
        Err(io::Error::new(
            self.kind,
            "rc9 explicitly injected filesystem failure",
        ))
    }
}

pub(super) fn inspect(entry: &TypeEntry) -> bool {
    let path = entry.path();
    let file_type = entry
        .file_type()
        .unwrap_or_else(|error| panic!("inspect {}: {error}", path.display()));
    file_type.is_dir()
}
