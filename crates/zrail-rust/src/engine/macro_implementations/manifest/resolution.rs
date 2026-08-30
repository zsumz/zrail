//! External implementation dependencies require one validated, content-bound Cargo lock graph.

use crate::cargo::{DependencySource, Package, ResolvedCargoGraph};

use super::CheckError;

pub(super) fn validate(
    packages: &[&Package],
    graph: Option<&ResolvedCargoGraph>,
    lock_bytes: Option<&[u8]>,
) -> Result<(), CheckError> {
    let external = packages.iter().any(|package| {
        package.dependencies.iter().any(|dependency| {
            matches!(
                dependency.source,
                DependencySource::Registry { .. } | DependencySource::Git { .. }
            )
        })
    });
    if !external {
        return Ok(());
    }
    let graph = graph.ok_or_else(|| {
        CheckError::from_message(
            "repository macro implementation with external dependencies requires Cargo.lock",
        )
    })?;
    let bytes = lock_bytes.ok_or_else(|| {
        CheckError::from_message(
            "repository macro implementation requires a captured regular Cargo.lock",
        )
    })?;
    if zrail_core::sha256_hex(bytes) != graph.lock_sha256() {
        return Err(CheckError::from_message(
            "repository macro Cargo.lock changed after exact dependency resolution",
        ));
    }
    for package in packages {
        for dependency in &package.dependencies {
            graph
                .manifest_dependency(package, dependency)
                .map_err(|error| {
                    CheckError::from_message(format!(
                        "repository macro dependency resolution: {error}"
                    ))
                })?;
        }
    }
    // The validated whole lock is part of the implementation input digest. Its
    // concrete names, versions, sources, checksums, and transitive edges are bound.
    Ok(())
}
