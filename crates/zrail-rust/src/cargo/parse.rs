//! Strict Cargo manifest projection without executing Cargo or build scripts.

use std::{collections::BTreeSet, error::Error, fmt, path::Path};

use toml::Value;
use zrail_core::input::read_text;

use crate::inventory::RepositoryInventory;

use super::{
    dependencies::{
        collect_dependencies, collect_dependency_paths, workspace_dependencies,
        workspace_dependency_paths,
    },
    model::{CargoWorkspace, Package},
    targets::collect_target_roots,
    workspace::{
        excluded_member, expand_implicit_members, expand_members, normalized_directory,
        workspace_excludes, workspace_members, workspace_package_edition,
    },
};

const MAX_TOTAL_MANIFEST_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CargoModelError(pub(super) String);

impl fmt::Display for CargoModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CargoModelError {}

pub(crate) fn load_cargo_workspace(
    inventory: &RepositoryInventory,
) -> Result<CargoWorkspace, CargoModelError> {
    let root_manifest = inventory.root.join("Cargo.toml");
    let mut manifest_bytes = 0;
    let root = read_manifest_counted(&root_manifest, &mut manifest_bytes)?;
    let workspace_dependencies = workspace_dependencies(&root).map_err(CargoModelError)?;
    let workspace_dependency_paths = workspace_dependency_paths(&root).map_err(CargoModelError)?;
    let workspace_edition = workspace_package_edition(&root)?;
    let member_patterns = workspace_members(&root)?;
    let exclude_patterns = workspace_excludes(&root)?;
    let root_package = package_name(&root)?.is_some();
    if !root_package && root.get("workspace").is_none() {
        return Err(CargoModelError(
            "root Cargo.toml requires a [package] or [workspace] table".into(),
        ));
    }
    let mut packages = Vec::new();
    for manifest in &inventory.manifest_paths {
        let value = if manifest == &root_manifest {
            root.clone()
        } else {
            read_manifest_counted(manifest, &mut manifest_bytes)?
        };
        let Some(name) = package_name(&value)? else {
            continue;
        };
        let directory = normalized_directory(&inventory.root, manifest)?;
        if directory != "." && excluded_member(&directory, &exclude_patterns) {
            continue;
        }
        packages.push(Package {
            name,
            directory,
            dependencies: collect_dependencies(&value, &workspace_dependencies)
                .map_err(CargoModelError)?,
            dependency_paths: collect_dependency_paths(&value, &workspace_dependency_paths)
                .map_err(CargoModelError)?,
            targets: collect_target_roots(
                &value,
                manifest.parent().unwrap_or(&inventory.root),
                workspace_edition.as_deref(),
            )
            .map_err(CargoModelError)?,
        });
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    ensure_unique_packages(&packages)?;
    let mut observed_members = packages
        .iter()
        .map(|package| package.directory.clone())
        .collect::<Vec<_>>();
    observed_members.sort();
    let declared_members = expand_members(&member_patterns, &observed_members, root_package)?;
    let declared_members = expand_implicit_members(declared_members, &packages, &exclude_patterns)?;
    Ok(CargoWorkspace {
        declared_members,
        observed_members,
        packages,
    })
}

fn read_manifest_counted(path: &Path, total: &mut usize) -> Result<Value, CargoModelError> {
    let source = read_text(path).map_err(CargoModelError)?;
    *total = total
        .checked_add(source.len())
        .ok_or_else(|| CargoModelError("Cargo manifest byte count overflowed".into()))?;
    if *total > MAX_TOTAL_MANIFEST_BYTES {
        return Err(CargoModelError(format!(
            "Cargo manifests exceed the {MAX_TOTAL_MANIFEST_BYTES}-byte total safety limit"
        )));
    }
    source
        .parse::<Value>()
        .map_err(|error| CargoModelError(format!("parse {}: {error}", path.display())))
}

fn package_name(value: &Value) -> Result<Option<String>, CargoModelError> {
    let Some(package) = value.get("package") else {
        return Ok(None);
    };
    let package = package
        .as_table()
        .ok_or_else(|| CargoModelError("Cargo [package] must be a table".into()))?;
    let name = package
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| CargoModelError("Cargo [package] requires string name".into()))?;
    Ok(Some(name.to_owned()))
}

fn ensure_unique_packages(packages: &[Package]) -> Result<(), CargoModelError> {
    let mut names = BTreeSet::new();
    for package in packages {
        if !names.insert(&package.name) {
            return Err(CargoModelError(format!(
                "duplicate Cargo package name {:?}",
                package.name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "parse_test.rs"]
mod parse_test;
