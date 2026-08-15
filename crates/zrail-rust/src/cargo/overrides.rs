//! Unsupported Cargo resolution indirection is retained as fail-closed evidence.

use std::{collections::BTreeSet, fs, path::Path};

use toml::Value;
use zrail_core::input::read_text;

use crate::inventory::{RepositoryEntryKind, RepositoryInventory};

use super::model::{CargoResolutionOverride, DependencySource, Package};

pub(super) fn manifest(value: &Value, path: &str, overrides: &mut Vec<CargoResolutionOverride>) {
    for (key, surface) in [
        ("patch", "manifest [patch] source override"),
        ("replace", "manifest [replace] source override"),
    ] {
        if value.get(key).is_some() {
            overrides.push(CargoResolutionOverride {
                path: path.into(),
                surface: surface.into(),
            });
        }
    }
}

pub(super) fn configuration(inventory: &RepositoryInventory) -> Vec<CargoResolutionOverride> {
    let mut overrides = Vec::new();
    let mut candidates = inventory
        .entries
        .iter()
        .filter(|entry| cargo_config_path(&entry.relative))
        .map(|entry| (entry.relative.clone(), entry.absolute.clone(), entry.kind))
        .collect::<Vec<_>>();
    add_root_configs(inventory, &mut candidates, &mut overrides);
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.dedup_by(|left, right| left.0 == right.0);
    overrides.extend(
        candidates
            .into_iter()
            .flat_map(|(relative, absolute, kind)| {
                if root_cargo_config_path(&relative) {
                    inspect_config(&relative, &absolute, kind)
                } else {
                    vec![resolution_override(
                        &relative,
                        "nested Cargo configuration has invocation-dependent resolution scope",
                    )]
                }
            }),
    );
    overrides
}

fn add_root_configs(
    inventory: &RepositoryInventory,
    candidates: &mut Vec<(String, std::path::PathBuf, RepositoryEntryKind)>,
    overrides: &mut Vec<CargoResolutionOverride>,
) {
    let cargo_directory = inventory.root.join(".cargo");
    match fs::symlink_metadata(&cargo_directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(_error) => {
            overrides.push(resolution_override(
                ".cargo",
                "Cargo configuration directory cannot be inspected safely",
            ));
            return;
        }
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            overrides.push(resolution_override(
                ".cargo",
                "Cargo configuration directory is not a repository-local directory",
            ));
            return;
        }
        Ok(_) => {}
    }
    for relative in [".cargo/config", ".cargo/config.toml"] {
        if candidates.iter().any(|candidate| candidate.0 == relative) {
            continue;
        }
        let absolute = inventory.root.join(relative);
        if let Ok(metadata) = fs::symlink_metadata(&absolute) {
            let kind = if metadata.file_type().is_symlink() {
                RepositoryEntryKind::Symlink
            } else if metadata.is_file() {
                RepositoryEntryKind::File
            } else {
                RepositoryEntryKind::Directory
            };
            candidates.push((relative.into(), absolute, kind));
        }
    }
}

pub(super) fn named_registries(packages: &[Package]) -> Vec<CargoResolutionOverride> {
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
                Some(CargoResolutionOverride {
                    path: package.manifest_path(),
                    surface: format!(
                        "named Cargo registry {registry:?} has no attested index mapping"
                    ),
                })
            })
        })
        .collect()
}

fn inspect_config(
    relative: &str,
    absolute: &Path,
    kind: RepositoryEntryKind,
) -> Vec<CargoResolutionOverride> {
    if kind != RepositoryEntryKind::File {
        return vec![resolution_override(
            relative,
            "Cargo configuration is not an attestable regular file",
        )];
    }
    let source = match read_text(absolute) {
        Ok(source) => source,
        Err(_error) => {
            return vec![resolution_override(
                relative,
                "Cargo configuration cannot be read as bounded UTF-8",
            )];
        }
    };
    let value = match source.parse::<Value>() {
        Ok(value) => value,
        Err(error) => {
            return vec![resolution_override(
                relative,
                &format!("Cargo configuration cannot be parsed: {error}"),
            )];
        }
    };
    config_surfaces(&value)
        .into_iter()
        .map(|surface| resolution_override(relative, surface))
        .collect()
}

fn config_surfaces(value: &Value) -> BTreeSet<&'static str> {
    let mut surfaces = BTreeSet::new();
    for (key, surface) in [
        ("paths", "Cargo config paths override"),
        ("source", "Cargo config source mapping or replacement"),
        ("patch", "Cargo config patch override"),
        ("replace", "Cargo config replacement override"),
    ] {
        if value.get(key).is_some() {
            surfaces.insert(surface);
        }
    }
    if value
        .get("registries")
        .and_then(Value::as_table)
        .is_some_and(|registries| {
            registries.values().any(|registry| {
                registry
                    .as_table()
                    .is_some_and(|registry| registry.contains_key("index"))
            })
        })
    {
        surfaces.insert("Cargo config named registry mapping");
    }
    if value
        .get("registry")
        .and_then(Value::as_table)
        .is_some_and(|registry| registry.contains_key("default") || registry.contains_key("index"))
    {
        surfaces.insert("Cargo config default registry mapping");
    }
    surfaces
}

fn cargo_config_path(relative: &str) -> bool {
    root_cargo_config_path(relative)
        || relative.ends_with("/.cargo/config")
        || relative.ends_with("/.cargo/config.toml")
}

fn root_cargo_config_path(relative: &str) -> bool {
    relative == ".cargo/config" || relative == ".cargo/config.toml"
}

fn resolution_override(path: &str, surface: &str) -> CargoResolutionOverride {
    CargoResolutionOverride {
        path: path.into(),
        surface: surface.into(),
    }
}

#[cfg(test)]
#[path = "overrides_test.rs"]
mod overrides_test;
