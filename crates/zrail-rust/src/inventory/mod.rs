//! Bounded, deterministic repository inventory.

mod classify;
mod exclusions;
mod io;
mod scan;
#[cfg(test)]
#[path = "../../tests/rc9_declarations/retired_fault_io.rs"]
pub(crate) mod test_faults;
mod traverse;
mod types;

pub(crate) use classify::{FileClass, classify_path, under_root};
#[cfg(test)]
pub(crate) use io::DirectoryIo;
#[cfg(test)]
pub(crate) use scan::inventory_cargo_repository;
pub(crate) use scan::{
    inventory_repository, inventory_selected_cargo_repository, load_referenced_source,
};
#[cfg(test)]
pub(crate) use traverse::scan_repository_using;
pub(crate) use traverse::{scan_file_entries, skip_directory};
pub(crate) use types::{RepositoryEntry, RepositoryEntryKind, RepositoryInventory, RustSourceFile};
