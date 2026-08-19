//! Bounded, deterministic repository inventory.

mod classify;
mod exclusions;
mod scan;
mod types;

pub(crate) use classify::{FileClass, classify_path, under_root};
pub(crate) use scan::{inventory_cargo_repository, inventory_repository};
pub(crate) use types::{RepositoryEntry, RepositoryEntryKind, RepositoryInventory, RustSourceFile};
