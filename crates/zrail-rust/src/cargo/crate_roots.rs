//! External crate roots require explicit Cargo naming or reviewed contract attestation.

use std::collections::BTreeMap;

use zrail_core::CrateRootContract;

use super::{CargoWorkspace, CrateRootAuthority, DependencySource, rust_crate_root};

pub(crate) fn apply_attestations(cargo: &mut CargoWorkspace, attestations: &[CrateRootContract]) {
    let attestations = attestations
        .iter()
        .map(|attestation| (attestation.package.as_str(), attestation))
        .collect::<BTreeMap<_, _>>();
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
        let Some(attestation) = attestations.get(dependency.name.as_str()) else {
            dependency.crate_root = rust_crate_root(&dependency.name);
            continue;
        };
        dependency.crate_root = attestation.root.clone();
        dependency.crate_root_authority = CrateRootAuthority::Attested;
    }
}

#[cfg(test)]
#[path = "crate_roots_test.rs"]
mod crate_roots_test;
