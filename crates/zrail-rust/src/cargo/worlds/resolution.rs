//! Lower and upper Cargo feature closures converge to one package feature map.

use std::collections::{BTreeMap, BTreeSet};

use super::{FeaturePackageSelection, contexts};
use crate::cargo::{CargoWorkspace, PackageFeatureResolution};

pub(super) struct FeatureClosure {
    pub(super) resolved: BTreeMap<String, PackageFeatureResolution>,
    pub(super) requested: BTreeMap<String, BTreeSet<String>>,
}

pub(super) fn fixed_point(
    cargo: &CargoWorkspace,
    selections: &BTreeMap<String, FeaturePackageSelection>,
    split_contexts: &contexts::SplitContexts,
    include_ambiguous: bool,
) -> Result<FeatureClosure, String> {
    let mut requested = selections
        .iter()
        .map(|(package, selection)| {
            (
                package.clone(),
                selection.features.iter().cloned().collect(),
            )
        })
        .collect::<BTreeMap<String, BTreeSet<String>>>();
    loop {
        let resolved = resolve_packages(cargo, selections, &requested)?;
        let mut changed = false;
        for package in &cargo.packages {
            let source = &resolved[&package.name];
            for dependency in &package.dependencies {
                let Some(destination) = dependency.internal_package() else {
                    continue;
                };
                if dependency.optional && !source.enabled_dependencies.contains(&dependency.alias) {
                    continue;
                }
                if split_contexts.edge_is_split(package, dependency) && !include_ambiguous {
                    continue;
                }
                let destination_features = &cargo.package_features[destination];
                let target = requested.get_mut(destination).ok_or_else(|| {
                    format!("workspace dependency resolves to missing package {destination:?}")
                })?;
                if dependency.default_features
                    && destination_features.declared().contains("default")
                {
                    changed |= target.insert("default".into());
                }
                for feature in dependency.features.iter().chain(
                    source
                        .dependency_features
                        .get(&dependency.alias)
                        .into_iter()
                        .flatten(),
                ) {
                    changed |= target.insert(feature.clone());
                }
            }
        }
        if !changed {
            return Ok(FeatureClosure {
                resolved,
                requested,
            });
        }
    }
}

fn resolve_packages(
    cargo: &CargoWorkspace,
    selections: &BTreeMap<String, FeaturePackageSelection>,
    requested: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, PackageFeatureResolution>, String> {
    selections
        .iter()
        .map(|(package, selection)| {
            let features = requested[package].iter().cloned().collect::<Vec<_>>();
            cargo.package_features[package]
                .resolve_details(selection.default_features, &features)
                .map(|resolved| (package.clone(), resolved))
                .map_err(|error| format!("package {package:?}: {error}"))
        })
        .collect()
}
