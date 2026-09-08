//! Only trusted unit tests can substitute directory results; there is no CLI or environment hook.

use crate::inventory::DirectoryIo;
use std::{
    cell::Cell,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Fault {
    Directory,
    EntryFirst,
    EntryLast,
    Metadata,
}

pub(crate) struct Injected {
    pub(crate) path: PathBuf,
    pub(crate) fault: Fault,
    pub(crate) kind: io::ErrorKind,
    pub(crate) hits: Cell<usize>,
}

pub(crate) struct Entries {
    real: fs::ReadDir,
    pending: Option<io::ErrorKind>,
    first: bool,
}

impl Iterator for Entries {
    type Item = io::Result<fs::DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if let Some(kind) = self.pending.take() {
                return Some(Err(error(kind)));
            }
        }
        self.real
            .next()
            .or_else(|| self.pending.take().map(|kind| Err(error(kind))))
    }
}

impl DirectoryIo for Injected {
    type Entries = Entries;

    fn read_dir(&self, path: &Path) -> io::Result<Self::Entries> {
        let matches = path == self.path && !matches!(self.fault, Fault::Metadata);
        if matches {
            self.hits.set(self.hits.get() + 1);
            if matches!(self.fault, Fault::Directory) {
                return Err(error(self.kind));
            }
        }
        Ok(Entries {
            real: fs::read_dir(path)?,
            pending: matches.then_some(self.kind),
            first: matches!(self.fault, Fault::EntryFirst),
        })
    }

    fn symlink_metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
        if path == self.path && matches!(self.fault, Fault::Metadata) {
            self.hits.set(self.hits.get() + 1);
            return Err(error(self.kind));
        }
        fs::symlink_metadata(path)
    }
}

fn error(kind: io::ErrorKind) -> io::Error {
    io::Error::new(kind, "rc9 explicitly injected filesystem failure")
}
