//! Cargo workspace and dependency facts without executing Cargo.

mod dependencies;
mod dependency_spec;
mod model;
mod parse;
mod target_discovery;
mod target_fields;
mod targets;
mod workspace;

pub(crate) use model::{
    CargoTargetKind, CargoWorkspace, DependencyKind, DependencySource, Package,
};
pub(crate) use parse::load_cargo_workspace;
