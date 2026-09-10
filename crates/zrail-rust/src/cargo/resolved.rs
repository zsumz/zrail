//! Exact offline dependency identities are derived from Cargo.lock without running Cargo.

mod build;
mod git_source;
mod mapping;
mod raw;

use std::{collections::BTreeMap, path::Path};

use zrail_core::{Contract, CrateRootSource};

use crate::cargo::{CargoModelError, Package};
use build::build;

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

    pub(crate) fn all_packages(&self) -> impl Iterator<Item = &ResolvedPackageIdentity> {
        self.packages.keys()
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

#[cfg(test)]
#[path = "resolved_test.rs"]
mod resolved_test;
