//! Bounded, deterministic repository inventory.

mod classify;
mod exclusions;
mod scan;
mod types;

pub(crate) use classify::{FileClass, classify_path, under_root};
#[cfg(test)]
pub(crate) use scan::inventory_cargo_repository;
pub(crate) use scan::{inventory_repository, inventory_selected_cargo_repository};
pub(crate) use types::{RepositoryEntry, RepositoryEntryKind, RepositoryInventory, RustSourceFile};
