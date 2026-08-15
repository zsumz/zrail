//! Semantic architecture changes, independent of TOML text layout.

mod boundaries;
mod compare;
mod contract;
mod evidence;
mod lock;
mod model;
mod source;
mod support;
mod topology;
mod topology_policy;

pub use compare::compare_architecture;
pub use model::{ArchitectureChange, ChangeKind, DiffReport, DiffSummary};
