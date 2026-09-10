//! Complete lock graphs map every local node to an active workspace manifest.

use std::collections::BTreeMap;

use crate::cargo::{CargoModelError, Package};

use super::{
    ResolvedCargoGraph, ResolvedPackage, ResolvedPackageIdentity,
    raw::{RawGraph, RawPackageId},
};

pub(super) fn build(
    raw: RawGraph,
    workspace: &[Package],
    lock_sha256: String,
) -> Result<ResolvedCargoGraph, CargoModelError> {
    let directories = workspace
        .iter()
        .map(|package| (package.name.as_str(), package.directory.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut identities = BTreeMap::new();
    let mut workspace_identities = BTreeMap::new();
    for (raw_id, package) in &raw {
        let source = if let Some(source) = raw_id.source.as_deref() {
            source.to_owned()
        } else {
            let directory = directories.get(raw_id.name.as_str()).ok_or_else(|| {
                CargoModelError(format!(
                    "Cargo.lock local package {} {} has no active workspace manifest",
                    raw_id.name, raw_id.version
                ))
            })?;
            format!("path+{directory}")
        };
        let identity = ResolvedPackageIdentity {
            name: raw_id.name.clone(),
            version: raw_id.version.clone(),
            source,
            checksum: package.checksum.clone(),
        };
        if raw_id.source.is_none()
            && workspace_identities
                .insert(raw_id.name.clone(), identity.clone())
                .is_some()
        {
            return Err(CargoModelError(format!(
                "Cargo.lock maps workspace package {:?} to multiple local nodes",
                raw_id.name
            )));
        }
        identities.insert(raw_id.clone(), identity);
    }
    let packages = raw
        .into_iter()
        .map(|(raw_id, package)| {
            let identity = mapped(&identities, &raw_id)?;
            let dependencies = package
                .dependencies
                .iter()
                .map(|dependency| mapped(&identities, dependency))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((identity, ResolvedPackage { dependencies }))
        })
        .collect::<Result<BTreeMap<_, _>, CargoModelError>>()?;
    for package in workspace {
        if !workspace_identities.contains_key(&package.name) {
            return Err(CargoModelError(format!(
                "Cargo.lock contains no local node for active workspace package {:?}",
                package.name
            )));
        }
    }
    Ok(ResolvedCargoGraph {
        packages,
        workspace: workspace_identities,
        lock_sha256,
    })
}

fn mapped(
    identities: &BTreeMap<RawPackageId, ResolvedPackageIdentity>,
    raw: &RawPackageId,
) -> Result<ResolvedPackageIdentity, CargoModelError> {
    identities.get(raw).cloned().ok_or_else(|| {
        CargoModelError(format!(
            "Cargo.lock lost resolved identity for {} {}",
            raw.name, raw.version
        ))
    })
}
