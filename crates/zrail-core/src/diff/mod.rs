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

#[cfg(test)]
#[path = "authority_test.rs"]
mod authority_test;

#[cfg(test)]
#[path = "compare_fixture_test.rs"]
mod compare_fixture_test;

pub use compare::{compare_architecture, compare_architecture_checked};
pub use model::{ArchitectureChange, ChangeKind, DiffReport, DiffSummary};

#[cfg(test)]
#[path = "compare_checked_test.rs"]
mod compare_checked_test;

#[cfg(test)]
#[path = "size_policy_test.rs"]
mod size_policy_test;
