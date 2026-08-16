//! Unsupported Cargo authority surfaces are retained as fail-closed evidence.

use std::fs;

use toml::Value;

use crate::inventory::RepositoryInventory;

use super::model::{CargoAuthorityKind, CargoAuthoritySurface, DependencySource, Package};

pub(super) fn manifest(value: &Value, path: &str, surfaces: &mut Vec<CargoAuthoritySurface>) {
    for (key, surface) in [
        ("patch", "manifest [patch] source override"),
        ("replace", "manifest [replace] source override"),
    ] {
        if value.get(key).is_some() {
            surfaces.push(CargoAuthoritySurface {
                kind: CargoAuthorityKind::Resolution,
                path: path.into(),
                surface: surface.into(),
            });
        }
    }
}

pub(super) fn configuration(inventory: &RepositoryInventory) -> Vec<CargoAuthoritySurface> {
    let mut surfaces = Vec::new();
    let mut candidates = inventory
        .entries
        .iter()
        .filter(|entry| cargo_config_path(&entry.relative))
        .map(|entry| entry.relative.clone())
        .collect::<Vec<_>>();
    add_root_configs(inventory, &mut candidates, &mut surfaces);
    candidates.sort();
    candidates.dedup();
    surfaces.extend(candidates.into_iter().map(|relative| {
        if root_cargo_config_path(&relative) {
            repository_configuration(&relative)
        } else {
            resolution_override(
                &relative,
                "nested Cargo configuration has invocation-dependent resolution scope",
            )
        }
    }));
    surfaces
}

fn add_root_configs(
    inventory: &RepositoryInventory,
    candidates: &mut Vec<String>,
    surfaces: &mut Vec<CargoAuthoritySurface>,
) {
    let cargo_directory = inventory.root.join(".cargo");
    match fs::symlink_metadata(&cargo_directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(_error) => {
            surfaces.push(repository_configuration(".cargo"));
            return;
        }
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            surfaces.push(repository_configuration(".cargo"));
            return;
        }
        Ok(_) => {}
    }
    for relative in [".cargo/config", ".cargo/config.toml"] {
        if candidates.iter().any(|candidate| candidate == relative) {
            continue;
        }
        let absolute = inventory.root.join(relative);
        if fs::symlink_metadata(&absolute).is_ok() {
            candidates.push(relative.into());
        }
    }
}

pub(super) fn named_registries(packages: &[Package]) -> Vec<CargoAuthoritySurface> {
    packages
        .iter()
        .flat_map(|package| {
            package.dependencies.iter().filter_map(|dependency| {
                let DependencySource::Registry {
                    registry: Some(registry),
                    index: None,
                    ..
                } = &dependency.source
                else {
                    return None;
                };
                Some(CargoAuthoritySurface {
                    kind: CargoAuthorityKind::Resolution,
                    path: package.manifest_path(),
                    surface: format!(
                        "named Cargo registry {registry:?} has no attested index mapping"
                    ),
                })
            })
        })
        .collect()
}

fn cargo_config_path(relative: &str) -> bool {
    root_cargo_config_path(relative)
        || relative.ends_with("/.cargo/config")
        || relative.ends_with("/.cargo/config.toml")
}

fn root_cargo_config_path(relative: &str) -> bool {
    relative == ".cargo/config" || relative == ".cargo/config.toml"
}

fn resolution_override(path: &str, surface: &str) -> CargoAuthoritySurface {
    CargoAuthoritySurface {
        kind: CargoAuthorityKind::Resolution,
        path: path.into(),
        surface: surface.into(),
    }
}

fn repository_configuration(path: &str) -> CargoAuthoritySurface {
    CargoAuthoritySurface {
        kind: CargoAuthorityKind::RepositoryConfiguration,
        path: path.into(),
        surface: "repository-local Cargo configuration can alter qualification execution".into(),
    }
}

#[cfg(test)]
#[path = "overrides_test.rs"]
mod overrides_test;
