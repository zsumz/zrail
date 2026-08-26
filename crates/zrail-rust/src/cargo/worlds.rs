//! Complete workspace feature worlds are resolved without invoking Cargo.

use std::collections::{BTreeMap, BTreeSet};

use super::{CargoWorkspace, DependencyKind, PackageFeatureResolution};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FeatureWorldSpec {
    pub(crate) name: String,
    pub(crate) packages: Vec<FeaturePackageSelection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FeaturePackageSelection {
    pub(crate) package: String,
    pub(crate) default_features: bool,
    pub(crate) features: Vec<String>,
}

impl From<&zrail_core::CargoFeatureWorldContract> for FeatureWorldSpec {
    fn from(world: &zrail_core::CargoFeatureWorldContract) -> Self {
        Self {
            name: world.name.clone(),
            packages: world
                .packages
                .iter()
                .map(|package| FeaturePackageSelection {
                    package: package.package.clone(),
                    default_features: package.default_features,
                    features: package.features.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedFeatureWorld {
    pub(crate) name: String,
    pub(crate) packages: BTreeMap<String, ResolvedPackageFeatures>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedPackageFeatures {
    pub(crate) default_features: bool,
    pub(crate) selected: Vec<String>,
    pub(crate) active: Vec<String>,
}

pub(crate) fn resolve_feature_worlds(
    cargo: &CargoWorkspace,
    specs: &[FeatureWorldSpec],
) -> Result<Vec<ResolvedFeatureWorld>, String> {
    let mut names = BTreeSet::new();
    let mut worlds = Vec::new();
    for spec in specs {
        if !names.insert(spec.name.as_str()) {
            return Err(format!("duplicate Cargo feature world {:?}", spec.name));
        }
        worlds.push(resolve_world(cargo, spec)?);
    }
    Ok(worlds)
}

fn resolve_world(
    cargo: &CargoWorkspace,
    spec: &FeatureWorldSpec,
) -> Result<ResolvedFeatureWorld, String> {
    let selections = selections(cargo, spec)?;
    let without_ambiguous = fixed_point(cargo, &selections, false)
        .map_err(|error| format!("feature world {:?}: {error}", spec.name))?;
    let with_ambiguous = fixed_point(cargo, &selections, true)
        .map_err(|error| format!("feature world {:?}: {error}", spec.name))?;
    if without_ambiguous != with_ambiguous {
        return Err(format!(
            "feature world {:?} is not exact: target-conditional, build, or development dependency edges change active features",
            spec.name
        ));
    }
    let packages = selections
        .into_iter()
        .map(|(package, selection)| {
            let active = with_ambiguous[&package].active.iter().cloned().collect();
            (
                package,
                ResolvedPackageFeatures {
                    default_features: selection.default_features,
                    selected: selection.features,
                    active,
                },
            )
        })
        .collect();
    Ok(ResolvedFeatureWorld {
        name: spec.name.clone(),
        packages,
    })
}

fn selections(
    cargo: &CargoWorkspace,
    spec: &FeatureWorldSpec,
) -> Result<BTreeMap<String, FeaturePackageSelection>, String> {
    let expected = cargo
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut selected = BTreeMap::new();
    for package in &spec.packages {
        if !expected.contains(package.package.as_str()) {
            return Err(format!(
                "feature world {:?} selects unknown package {:?}",
                spec.name, package.package
            ));
        }
        let mut normalized = package.clone();
        normalized.features.sort();
        if normalized
            .features
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(format!(
                "feature world {:?} repeats a feature for package {:?}",
                spec.name, package.package
            ));
        }
        if selected
            .insert(package.package.clone(), normalized)
            .is_some()
        {
            return Err(format!(
                "feature world {:?} repeats package {:?}",
                spec.name, package.package
            ));
        }
    }
    let missing = expected
        .iter()
        .filter(|package| !selected.contains_key(**package))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "feature world {:?} must select every workspace package; missing {}",
            spec.name,
            missing.join(", ")
        ));
    }
    Ok(selected)
}

fn fixed_point(
    cargo: &CargoWorkspace,
    selections: &BTreeMap<String, FeaturePackageSelection>,
    include_ambiguous: bool,
) -> Result<BTreeMap<String, PackageFeatureResolution>, String> {
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
                let ambiguous =
                    dependency.target.is_some() || dependency.kind != DependencyKind::Normal;
                if ambiguous && !include_ambiguous {
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
            return Ok(resolved);
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

#[cfg(test)]
#[path = "worlds_test.rs"]
mod worlds_test;
