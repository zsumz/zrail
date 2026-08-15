//! Bounded, deterministic repository inventory.

mod classify;
mod exclusions;
mod scan;
mod types;

pub use classify::FileClass;
pub(crate) use classify::{classify_path, under_root};
pub(crate) use scan::inventory_cargo_repository;
pub use scan::{RepositoryInventoryError, inventory_repository};
pub(crate) use types::RepositoryEntry;
pub use types::{RepositoryEntryKind, RepositoryInventory, RustSourceFile};
