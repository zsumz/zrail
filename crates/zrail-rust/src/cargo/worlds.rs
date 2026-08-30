//! Complete workspace feature worlds are resolved without invoking Cargo.

use std::collections::{BTreeMap, BTreeSet};

use super::CargoWorkspace;

mod contexts;
mod diagnostics;
mod resolution;

use resolution::{FeatureClosure, fixed_point};

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
    let split_contexts = contexts::SplitContexts::new(cargo);
    let mut names = BTreeSet::new();
    let mut worlds = Vec::new();
    for spec in specs {
        if !names.insert(spec.name.as_str()) {
            return Err(format!("duplicate Cargo feature world {:?}", spec.name));
        }
        worlds.push(resolve_world(cargo, spec, &split_contexts)?);
    }
    Ok(worlds)
}

fn resolve_world(
    cargo: &CargoWorkspace,
    spec: &FeatureWorldSpec,
    split_contexts: &contexts::SplitContexts,
) -> Result<ResolvedFeatureWorld, String> {
    let selections = selections(cargo, spec)?;
    let without_ambiguous = fixed_point(cargo, &selections, split_contexts, false)
        .map_err(|error| format!("feature world {:?}: {error}", spec.name))?;
    let with_ambiguous = fixed_point(cargo, &selections, split_contexts, true)
        .map_err(|error| format!("feature world {:?}: {error}", spec.name))?;
    if without_ambiguous.resolved != with_ambiguous.resolved {
        return Err(diagnostics::non_convergent_world(
            cargo,
            spec,
            &without_ambiguous,
            &with_ambiguous,
            split_contexts,
        ));
    }
    if let Some(error) = diagnostics::nonempty_split_context(spec, &with_ambiguous, split_contexts)
    {
        return Err(error);
    }
    let packages = selections
        .into_iter()
        .map(|(package, selection)| {
            let active = with_ambiguous.resolved[&package]
                .active
                .iter()
                .cloned()
                .collect();
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

#[cfg(test)]
#[path = "worlds_test.rs"]
mod worlds_test;
