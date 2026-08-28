//! File contexts select packages before dependency roots become policy identities.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::{CrateRootAuthority, Package, rust_crate_root};

pub(super) fn external_roots(
    cargo: &crate::cargo::CargoWorkspace,
) -> BTreeMap<String, BTreeSet<String>> {
    cargo
        .packages
        .iter()
        .map(|package| {
            let roots = package
                .dependencies
                .iter()
                .filter(|dependency| {
                    dependency.crate_root_authority != CrateRootAuthority::Unresolved
                })
                .map(|dependency| rust_crate_root(&dependency.crate_root))
                .collect();
            (package.name.clone(), roots)
        })
        .collect()
}

pub(super) fn selected_packages<'a>(
    contexts: &BTreeMap<String, BTreeSet<String>>,
    packages: &BTreeMap<&'a str, &'a Package>,
    all: &'a [Package],
    file: &str,
) -> Vec<&'a Package> {
    contexts.get(file).map_or_else(
        || package_for_file(all, file).into_iter().collect(),
        |names| {
            names
                .iter()
                .filter_map(|name| packages.get(name.as_str()).copied())
                .collect()
        },
    )
}

pub(super) fn dependency_roots(
    packages: &[&Package],
    observed: &BTreeSet<String>,
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>) {
    let mut roots = BTreeMap::<String, BTreeSet<String>>::new();
    let mut overflowed = BTreeSet::new();
    for package in packages {
        for dependency in &package.dependencies {
            if dependency.crate_root_authority == CrateRootAuthority::Unresolved {
                continue;
            }
            let crate_root = rust_crate_root(&dependency.crate_root);
            if !observed.contains(&crate_root) || overflowed.contains(&crate_root) {
                continue;
            }
            let canonical = rust_crate_root(&dependency.name);
            let identities = roots.entry(crate_root.clone()).or_default();
            if identities.len() == super::MAX_IDENTITIES_PER_ROOT
                && !identities.contains(&canonical)
            {
                overflowed.insert(crate_root);
            } else {
                identities.insert(canonical);
            }
        }
    }
    (roots, overflowed)
}

fn package_for_file<'a>(packages: &'a [Package], file: &str) -> Option<&'a Package> {
    packages
        .iter()
        .filter(|package| package.contains_file(file))
        .max_by_key(|package| package.directory.len())
}
