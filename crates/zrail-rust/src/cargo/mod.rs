//! Cargo workspace and dependency facts without executing Cargo.

mod crate_roots;
mod dependencies;
mod dependency_spec;
mod model;
mod overrides;
mod parse;
mod target_discovery;
mod target_explicit;
mod target_fields;
mod targets;
mod workspace;

pub(crate) use crate_roots::apply_attestations;
pub(crate) use model::{
    CargoTargetKind, CargoWorkspace, CrateRootAuthority, DependencyKind, DependencySource, Package,
    rust_crate_root,
};
pub(crate) use parse::load_cargo_workspace;

#[cfg(test)]
pub(crate) use model::Dependency;
