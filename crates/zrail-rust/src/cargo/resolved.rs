//! Exact offline dependency identities are derived from Cargo.lock without running Cargo.

mod mapping;
mod raw;

use std::{collections::BTreeMap, path::Path};

use zrail_core::{Contract, CrateRootSource};

use crate::cargo::{CargoModelError, Package};
use raw::{RawGraph, RawPackageId};

/// One immutable Cargo.lock package identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ResolvedPackageIdentity {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) checksum: Option<String>,
}

impl ResolvedPackageIdentity {
    pub(crate) fn label(&self) -> String {
        format!(
            "{} {} ({}; checksum={})",
            self.name,
            self.version,
            self.source,
            self.checksum.as_deref().unwrap_or("none")
        )
    }
}

#[derive(Clone, Debug)]
struct ResolvedPackage {
    dependencies: Vec<ResolvedPackageIdentity>,
}

/// Complete package and edge graph parsed from one repository Cargo.lock.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedCargoGraph {
    packages: BTreeMap<ResolvedPackageIdentity, ResolvedPackage>,
    workspace: BTreeMap<String, ResolvedPackageIdentity>,
    lock_sha256: String,
}

impl ResolvedCargoGraph {
    pub(crate) fn load(
        root: &Path,
        workspace: &[Package],
    ) -> Result<Option<Self>, CargoModelError> {
        raw::load(root)?
            .map(|(raw, sha256)| build(raw, workspace, sha256))
            .transpose()
    }

    pub(crate) fn lookup(
        &self,
        package: &str,
        version: Option<&str>,
        source: Option<&str>,
    ) -> Result<&ResolvedPackageIdentity, String> {
        let matches = self
            .packages
            .keys()
            .filter(|identity| {
                identity.name == package
                    && version.is_none_or(|value| identity.version == value)
                    && source.is_none_or(|value| identity.source == value)
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [identity] => Ok(identity),
            [] => Err(format!(
                "Cargo.lock contains no package matching name {package:?}, version {version:?}, source {source:?}"
            )),
            _ => Err(format!(
                "Cargo.lock package selector name {package:?}, version {version:?}, source {source:?} is ambiguous across {} nodes: {}",
                matches.len(),
                matches
                    .iter()
                    .map(|identity| identity.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub(crate) fn dependencies(
        &self,
        package: &ResolvedPackageIdentity,
    ) -> &[ResolvedPackageIdentity] {
        self.packages
            .get(package)
            .map_or(&[], |node| node.dependencies.as_slice())
    }

    pub(crate) fn workspace_package(&self, name: &str) -> Option<&ResolvedPackageIdentity> {
        self.workspace.get(name)
    }

    pub(crate) fn lock_sha256(&self) -> &str {
        &self.lock_sha256
    }
}

pub(crate) fn validate_resolved_sources(
    graph: Option<&ResolvedCargoGraph>,
    contract: &Contract,
) -> Result<(), CargoModelError> {
    for source in contract
        .dependencies
        .crate_roots
        .iter()
        .map(|attestation| &attestation.source)
        .chain(
            contract
                .source
                .rust
                .item_macros
                .iter()
                .filter_map(|allowance| allowance.source.as_ref()),
        )
        .chain(
            contract
                .source
                .rust
                .macros
                .allow
                .iter()
                .filter_map(|allowance| allowance.source.as_ref()),
        )
    {
        let CrateRootSource::CargoLock {
            package,
            version,
            source,
        } = source
        else {
            continue;
        };
        let graph = graph.ok_or_else(|| {
            CargoModelError(format!(
                "Cargo.lock source selector for package {package:?} requires Cargo.lock"
            ))
        })?;
        graph
            .lookup(package, version.as_deref(), source.as_deref())
            .map_err(CargoModelError)?;
    }
    Ok(())
}

fn build(
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

#[cfg(test)]
#[path = "resolved_test.rs"]
mod resolved_test;
