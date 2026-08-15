//! Cargo workspace and dependency facts without executing Cargo.

mod dependencies;
mod model;
mod parse;
mod target_discovery;
mod targets;
mod workspace;

pub(crate) use model::{CargoTargetKind, CargoWorkspace, DependencyKind, Package};
pub(crate) use parse::load_cargo_workspace;
