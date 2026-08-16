//! External crate roots require explicit Cargo naming or reviewed contract attestation.

use zrail_core::{CrateRootContract, CrateRootSource};

use super::{CargoWorkspace, CrateRootAuthority, DependencySource, rust_crate_root};

pub(crate) fn apply_attestations(cargo: &mut CargoWorkspace, attestations: &[CrateRootContract]) {
    for dependency in cargo
        .packages
        .iter_mut()
        .flat_map(|package| &mut package.dependencies)
    {
        if dependency.crate_root_authority != CrateRootAuthority::Unresolved
            || !matches!(
                dependency.source,
                DependencySource::Registry { .. } | DependencySource::Git { .. }
            )
        {
            continue;
        }
        let Some(attestation) = attestations.iter().find(|attestation| {
            attestation_matches(attestation, &dependency.name, &dependency.source)
        }) else {
            dependency.crate_root = rust_crate_root(&dependency.name);
            continue;
        };
        dependency.crate_root = attestation.root.clone();
        dependency.crate_root_authority = CrateRootAuthority::Attested;
    }
}

pub(crate) fn attestation_matches(
    attestation: &CrateRootContract,
    package: &str,
    source: &DependencySource,
) -> bool {
    attestation.package == package && source_matches(&attestation.source, source)
}

pub(crate) fn source_matches(attested: &CrateRootSource, source: &DependencySource) -> bool {
    match (attested, source) {
        (
            CrateRootSource::Registry {
                registry: left_registry,
                index: left_index,
                requirement: left_requirement,
            },
            DependencySource::Registry {
                registry,
                index,
                requirement,
            },
        ) => left_registry == registry && left_index == index && left_requirement == requirement,
        (
            CrateRootSource::Git {
                repository: left_repository,
                branch: left_branch,
                tag: left_tag,
                rev: left_rev,
                requirement: left_requirement,
            },
            DependencySource::Git {
                repository,
                branch,
                tag,
                rev,
                requirement,
            },
        ) => {
            left_repository == repository
                && left_branch == branch
                && left_tag == tag
                && left_rev == rev
                && left_requirement == requirement
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "crate_roots_test.rs"]
mod crate_roots_test;
