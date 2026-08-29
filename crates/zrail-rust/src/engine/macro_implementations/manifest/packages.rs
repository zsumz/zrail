//! Bounded transitive closure of in-repository Cargo dependencies.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::{DependencySource, Package};

use super::CheckError;

const MAX_IMPLEMENTATION_PACKAGES: usize = 4_096;

pub(super) fn implementation_packages<'a>(
    packages: &'a [Package],
    root: &'a Package,
) -> Result<Vec<&'a Package>, CheckError> {
    let by_directory = packages
        .iter()
        .map(|package| (package.directory.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut selected = BTreeSet::new();
    let mut pending = BTreeSet::from([root.directory.clone()]);
    while let Some(directory) = pending.pop_first() {
        if !selected.insert(directory.clone()) {
            continue;
        }
        if selected.len() > MAX_IMPLEMENTATION_PACKAGES {
            return Err(CheckError::from_message(format!(
                "macro implementation exceeds the {MAX_IMPLEMENTATION_PACKAGES}-package safety limit"
            )));
        }
        let package = by_directory.get(directory.as_str()).ok_or_else(|| {
            CheckError::from_message(format!(
                "repository macro implementation package at {directory:?} is unavailable"
            ))
        })?;
        for dependency in &package.dependencies {
            let Some(target) = internal_directory(&dependency.source) else {
                continue;
            };
            let helper = by_directory.get(target).ok_or_else(|| {
                CheckError::from_message(format!(
                    "repository macro implementation dependency {:?} from package {:?} targets unavailable internal path {target:?}",
                    dependency.alias, package.name
                ))
            })?;
            if helper.name != dependency.name {
                return Err(CheckError::from_message(format!(
                    "repository macro implementation dependency {:?} from package {:?} names {:?}, but internal path {target:?} contains {:?}",
                    dependency.alias, package.name, dependency.name, helper.name
                )));
            }
            if !selected.contains(target) {
                pending.insert(target.into());
            }
        }
    }
    selected
        .iter()
        .map(|directory| {
            by_directory
                .get(directory.as_str())
                .copied()
                .ok_or_else(|| {
                    CheckError::from_message(format!(
                        "repository macro implementation package at {directory:?} is unavailable"
                    ))
                })
        })
        .collect()
}

fn internal_directory(source: &DependencySource) -> Option<&str> {
    match source {
        DependencySource::WorkspaceMember { directory, .. } => Some(directory),
        DependencySource::RepositoryPath { path, .. } => Some(path),
        DependencySource::Registry { .. } | DependencySource::Git { .. } => None,
    }
}

#[cfg(test)]
#[path = "packages_test.rs"]
mod packages_test;
