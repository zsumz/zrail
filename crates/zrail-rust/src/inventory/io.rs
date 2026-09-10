//! Statically dispatched filesystem operations let trusted tests inject exact traversal failures.

use std::{fs, io, path::Path};

pub(crate) trait DirectoryIo {
    type Entries: Iterator<Item = io::Result<fs::DirEntry>>;

    fn read_dir(&self, path: &Path) -> io::Result<Self::Entries>;
    fn symlink_metadata(&self, path: &Path) -> io::Result<fs::Metadata>;
}

pub(super) struct Filesystem;

impl DirectoryIo for Filesystem {
    type Entries = fs::ReadDir;

    fn read_dir(&self, path: &Path) -> io::Result<Self::Entries> {
        fs::read_dir(path)
    }

    fn symlink_metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
        fs::symlink_metadata(path)
    }
}
