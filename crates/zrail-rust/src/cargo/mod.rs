//! Cargo workspace and dependency facts without executing Cargo.

mod crate_roots;
mod dependencies;
mod dependency_spec;
mod model;
mod overrides;
mod parse;
mod resolved;
mod target_discovery;
mod target_explicit;
mod target_fields;
mod targets;
mod workspace;
mod workspace_plan;

#[cfg(test)]
#[path = "resolved_git_test.rs"]
mod resolved_git_test;

pub(crate) use crate_roots::{apply_attestations, attestation_matches, source_matches};
pub(crate) use model::{
    CargoAuthorityKind, CargoTargetKind, CargoWorkspace, CrateRootAuthority, Dependency,
    DependencyKind, DependencySource, Package, rust_crate_root,
};
pub(crate) use parse::{CargoModelError, load_cargo_workspace};
pub(crate) use resolved::{ResolvedCargoGraph, ResolvedPackageIdentity, validate_resolved_sources};
