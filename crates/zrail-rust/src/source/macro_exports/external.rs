//! External glob exports come only from checksum-matched offline registry archives.

#[path = "external/archive.rs"]
mod archive;
#[path = "external/exports.rs"]
mod exports;

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::{CargoWorkspace, Dependency, DependencySource, ResolvedCargoGraph};

use super::{ModuleDraft, normalize};
use exports::{ModuleExports, PackageExports};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ExternalModule {
    pub(super) consumer: String,
    pub(super) crate_root: String,
    pub(super) path: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ExternalKey {
    consumer: String,
    crate_root: String,
}

#[derive(Debug, Default)]
pub(super) struct ExternalMacroCatalog {
    packages: BTreeMap<ExternalKey, Result<PackageExports, String>>,
}

impl ExternalMacroCatalog {
    pub(super) fn collect(
        cargo: &CargoWorkspace,
        resolved: Option<&ResolvedCargoGraph>,
        drafts: &BTreeMap<super::LogicalModule, ModuleDraft>,
    ) -> Self {
        let requested = requested_dependencies(cargo, drafts);
        let packages = requested
            .into_iter()
            .map(|(key, dependency)| {
                let package = cargo
                    .packages
                    .iter()
                    .find(|package| package.name == key.consumer)
                    .ok_or_else(|| format!("Cargo package {:?} is unavailable", key.consumer));
                let analyzed =
                    package.and_then(|package| analyze_dependency(package, &dependency, resolved));
                (key, analyzed)
            })
            .collect();
        Self { packages }
    }

    pub(super) fn module(&self, module: &ExternalModule) -> Result<ModuleExports, String> {
        let key = ExternalKey {
            consumer: module.consumer.clone(),
            crate_root: module.crate_root.clone(),
        };
        self.packages
            .get(&key)
            .ok_or_else(|| {
                format!(
                    "external macro archive for dependency root {:?} was not selected for analysis",
                    module.crate_root
                )
            })?
            .as_ref()
            .map(|package| package.module(&module.path))
            .map_err(Clone::clone)
    }
}

fn analyze_dependency(
    package: &crate::cargo::Package,
    dependency: &Dependency,
    resolved: Option<&ResolvedCargoGraph>,
) -> Result<PackageExports, String> {
    if !matches!(dependency.source, DependencySource::Registry { .. }) {
        return Err(format!(
            "external macro dependency {:?} is not a checksum-bound registry source",
            dependency.crate_root
        ));
    }
    let resolved = resolved
        .ok_or_else(|| "Cargo.lock is unavailable for external macro export analysis".to_owned())?;
    let identity = resolved.manifest_dependency(package, dependency)?;
    let archive = archive::load(&identity)?;
    PackageExports::analyze(&archive)
}

fn requested_dependencies(
    cargo: &CargoWorkspace,
    drafts: &BTreeMap<super::LogicalModule, ModuleDraft>,
) -> BTreeMap<ExternalKey, Dependency> {
    let mut roots = BTreeSet::new();
    for (module, draft) in drafts {
        for glob in &draft.globs {
            let Some(root) = glob
                .target
                .split("::")
                .find(|segment| !segment.is_empty())
                .map(normalize)
            else {
                continue;
            };
            roots.insert((module.domain.package.clone(), root.to_owned()));
        }
    }
    let mut requested = BTreeMap::new();
    for (consumer, crate_root) in roots {
        let Some(package) = cargo
            .packages
            .iter()
            .find(|package| package.name == consumer)
        else {
            continue;
        };
        let matches = package
            .dependencies
            .iter()
            .filter(|dependency| normalize(&dependency.crate_root) == crate_root)
            .collect::<Vec<_>>();
        if let [dependency] = matches.as_slice()
            && matches!(
                dependency.source,
                DependencySource::Registry { .. } | DependencySource::Git { .. }
            )
        {
            requested.insert(
                ExternalKey {
                    consumer,
                    crate_root,
                },
                (*dependency).clone(),
            );
        }
    }
    requested
}
