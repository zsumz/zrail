//! Repository inventory facts independent of Rust parsing.

use std::path::PathBuf;

use super::FileClass;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum RepositoryEntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositoryEntry {
    pub(crate) relative: String,
    pub(crate) absolute: PathBuf,
    pub(crate) kind: RepositoryEntryKind,
}

#[derive(Clone, Debug)]
pub(crate) struct RepositoryInventory {
    pub(crate) root: PathBuf,
    pub(crate) entries: Vec<RepositoryEntry>,
    pub(crate) rust_files: Vec<RustSourceFile>,
    pub(crate) manifest_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct RustSourceFile {
    pub(crate) relative: String,
    pub(crate) class: FileClass,
    pub(crate) source: String,
    pub(crate) lines: usize,
}
