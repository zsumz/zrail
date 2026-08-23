//! Strict Cargo manifest projection without executing Cargo or build scripts.

use std::{collections::BTreeSet, error::Error, fmt, path::Path};

use toml::Value;
use zrail_core::read_text;

use crate::inventory::RepositoryInventory;

use super::{
    dependencies::{collect_dependencies, workspace_dependencies},
    model::{CargoWorkspace, ManifestScope, Package},
    overrides,
    target_discovery::package_edition,
    targets::collect_target_roots,
    workspace::{
        excluded_member, expand_implicit_members, expand_members, normalized_directory,
        resolve_workspace_dependencies, workspace_package_edition,
    },
    workspace_plan,
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
    let workspace_edition = workspace_package_edition(&root)?;
    let root_package = package_name(&root)?.is_some();
    if !root_package && root.get("workspace").is_none() {
        return Err(CargoModelError(
            "root Cargo.toml requires a [package] or [workspace] table".into(),
        ));
    }
    let plan = workspace_plan::build(
        inventory,
        &root_manifest,
        root,
        &workspace_dependencies,
        root_package,
        &mut manifest_bytes,
    )?;
    let mut packages = Vec::new();
    let mut authority_surfaces = Vec::new();
    for manifest in &plan.selected_manifests {
        let value = plan.value(manifest)?;
        let directory = normalized_directory(&inventory.root, manifest)?;
        let manifest_path = if directory == "." {
            "Cargo.toml".into()
        } else {
            format!("{directory}/Cargo.toml")
        };
        overrides::manifest(value, &manifest_path, &mut authority_surfaces);
        let Some(name) = package_name(value)? else {
            continue;
        };
        packages.push(Package {
            name,
            edition: package_edition(
                value
                    .get("package")
                    .and_then(Value::as_table)
                    .ok_or_else(|| CargoModelError("Cargo package requires [package]".into()))?,
                workspace_edition.as_deref(),
            )
            .map_err(CargoModelError)?,
            dependencies: collect_dependencies(value, &workspace_dependencies, &directory)
                .map_err(CargoModelError)?,
            directory,
            targets: collect_target_roots(
                value,
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
        .filter(|package| {
            package.directory == "." || !excluded_member(&package.directory, &plan.exclude_patterns)
        })
        .map(|package| package.directory.clone())
        .collect::<Vec<_>>();
    observed_members.extend(plan.observed_extras.iter().cloned());
    observed_members.sort();
    observed_members.dedup();
    let declared_members = expand_members(&plan.member_patterns, &observed_members, root_package)?;
    let declared_members =
        expand_implicit_members(declared_members, &packages, &plan.exclude_patterns)?;
    resolve_workspace_dependencies(&mut packages, &declared_members)?;
    authority_surfaces.extend(overrides::named_registries(&packages));
    authority_surfaces.extend(overrides::configuration(inventory));
    authority_surfaces.sort();
    authority_surfaces.dedup();
    Ok(CargoWorkspace {
        declared_members,
        observed_members,
        packages,
        authority_surfaces,
        manifest_scopes: plan
            .selected_manifests
            .iter()
            .map(|manifest| normalized_directory(&inventory.root, manifest))
            .map(|directory| directory.map(|directory| (directory, ManifestScope::Active)))
            .chain(
                plan.observed_extras
                    .into_iter()
                    .map(|directory| Ok((directory, ManifestScope::ObservedExtra))),
            )
            .chain(
                plan.excluded_boundaries
                    .into_iter()
                    .map(|directory| Ok((directory, ManifestScope::IgnoredExcluded))),
            )
            .chain(
                plan.nested_workspace_boundaries
                    .into_iter()
                    .map(|directory| Ok((directory, ManifestScope::IgnoredNestedWorkspace))),
            )
            .collect::<Result<_, _>>()?,
    })
}

pub(super) fn read_manifest_counted(
    path: &Path,
    total: &mut usize,
) -> Result<Value, CargoModelError> {
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

pub(super) fn package_name(value: &Value) -> Result<Option<String>, CargoModelError> {
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
