//! Exact workspace-member resolution for repository path dependencies.

use std::collections::BTreeMap;

use crate::cargo::{
    model::{CrateRootAuthority, DependencySource, Package},
    parse::CargoModelError,
};

pub(in crate::cargo) fn resolve_workspace_dependencies(
    packages: &mut [Package],
    declared_members: &[String],
) -> Result<(), CargoModelError> {
    let targets = packages
        .iter()
        .map(|package| {
            (
                package.directory.clone(),
                (
                    package.name.clone(),
                    declared_members.contains(&package.directory),
                    package.library_crate_root().map(str::to_owned),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for package in packages {
        for dependency in &mut package.dependencies {
            let (path, requirement) = match &dependency.source {
                DependencySource::RepositoryPath { path, requirement } => {
                    (path.clone(), requirement.clone())
                }
                _ => continue,
            };
            let Some((target_name, member, crate_root)) = targets.get(&path) else {
                continue;
            };
            if dependency.name != *target_name {
                return Err(CargoModelError(format!(
                    "dependency {:?} in package {:?} names package {:?}, but path {:?} contains {:?}",
                    dependency.alias, package.name, dependency.name, path, target_name
                )));
            }
            if *member {
                dependency.source = DependencySource::WorkspaceMember {
                    directory: path.clone(),
                    requirement,
                };
            }
            let Some(crate_root) = crate_root else {
                return Err(CargoModelError(format!(
                    "dependency {:?} in package {:?} targets path {:?}, which has no library target",
                    dependency.alias, package.name, path
                )));
            };
            if !dependency.explicit_package {
                dependency.crate_root.clone_from(crate_root);
                dependency.crate_root_authority = CrateRootAuthority::InspectedLibrary;
            }
        }
    }
    Ok(())
}
