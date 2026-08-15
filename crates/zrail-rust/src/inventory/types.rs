//! Repository inventory facts independent of Rust parsing.

use std::path::PathBuf;

use super::FileClass;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RepositoryEntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryEntry {
    pub relative: String,
    pub absolute: PathBuf,
    pub kind: RepositoryEntryKind,
}

#[derive(Clone, Debug)]
pub struct RepositoryInventory {
    pub root: PathBuf,
    pub entries: Vec<RepositoryEntry>,
    pub rust_files: Vec<RustSourceFile>,
    pub manifest_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct RustSourceFile {
    pub relative: String,
    pub absolute: PathBuf,
    pub class: FileClass,
    pub source: String,
    pub lines: usize,
}
