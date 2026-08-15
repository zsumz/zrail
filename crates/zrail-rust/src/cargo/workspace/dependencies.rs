//! Exact workspace-member resolution for repository path dependencies.

use std::collections::BTreeMap;

use crate::cargo::{
    model::{DependencySource, Package},
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
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for package in packages {
        for dependency in &mut package.dependencies {
            let DependencySource::RepositoryPath { path, requirement } = &dependency.source else {
                continue;
            };
            let Some((target_name, member)) = targets.get(path) else {
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
                    requirement: requirement.clone(),
                };
            }
        }
    }
    Ok(())
}
