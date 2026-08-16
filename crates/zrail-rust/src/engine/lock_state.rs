//! Candidate lock generation and exact lock drift findings.

use std::{collections::BTreeMap, path::Path};

use zrail_core::{
    LockFile, LockedDependency, LockedDependencyKind, LockedDependencyScope,
    LockedDependencySource, LockedPackage, LockedRatchet,
};

use crate::cargo::{DependencyKind, DependencySource};

use super::{
    CheckError,
    model::{RepositoryModel, resolve},
};

pub(super) fn candidate_lock(model: &RepositoryModel) -> Result<LockFile, CheckError> {
    let mut lock = LockFile::new(&model.bundle.sha256);
    lock.generated = crate::rules::generated::locked_sources(
        &model.inventory.root,
        &model.bundle.contract.source.rust.generated,
    );
    lock.gates = super::gates::locked(model)?;
    lock.macro_implementations = super::macro_implementations::locked(model)?;
    for package in &model.cargo.packages {
        let dependencies = package
            .dependencies
            .iter()
            .map(|dependency| LockedDependency {
                alias: Some(dependency.alias.clone()),
                name: dependency.name.clone(),
                crate_root: (dependency.crate_root_authority
                    != crate::cargo::CrateRootAuthority::Unresolved)
                    .then(|| dependency.crate_root.clone()),
                kind: locked_kind(dependency.kind),
                scope: if dependency.internal_package().is_some() {
                    LockedDependencyScope::Internal
                } else {
                    LockedDependencyScope::External
                },
                target: dependency.target.clone(),
                optional: Some(dependency.optional),
                default_features: Some(dependency.default_features),
                features: dependency.features.clone(),
                source: Some(locked_source(&dependency.source)),
            })
            .collect();
        lock.packages.push(LockedPackage {
            name: package.name.clone(),
            dependencies,
        });
    }
    let sources = model
        .source
        .files
        .iter()
        .map(|file| (file.relative.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    for ratchet in &model.bundle.contract.ratchets {
        if let Some(value) = sources
            .get(ratchet.target.as_str())
            .and_then(|file| ratchet_value(&ratchet.rule, file))
            .filter(|value| *value > 0)
        {
            lock.ratchets.push(LockedRatchet {
                rule: ratchet.rule.clone(),
                target: ratchet.target.clone(),
                value,
            });
        }
    }
    lock.canonicalize()
        .map_err(|error| CheckError::from_message(error.to_string()))?;
    Ok(lock)
}

fn locked_source(source: &DependencySource) -> LockedDependencySource {
    match source {
        DependencySource::WorkspaceMember {
            directory,
            requirement,
        } => LockedDependencySource::WorkspaceMember {
            directory: directory.clone(),
            requirement: requirement.clone(),
        },
        DependencySource::RepositoryPath { path, requirement } => {
            LockedDependencySource::RepositoryPath {
                path: path.clone(),
                requirement: requirement.clone(),
            }
        }
        DependencySource::Registry {
            registry,
            index,
            requirement,
        } => LockedDependencySource::Registry {
            registry: registry.clone(),
            index: index.clone(),
            requirement: requirement.clone(),
        },
        DependencySource::Git {
            repository,
            branch,
            tag,
            rev,
            requirement,
        } => LockedDependencySource::Git {
            repository: repository.clone(),
            branch: branch.clone(),
            tag: tag.clone(),
            rev: rev.clone(),
            requirement: requirement.clone(),
        },
    }
}

fn ratchet_value(rule: &str, file: &crate::source::RustFileFacts) -> Option<usize> {
    match rule {
        "rust.file-size" => Some(file.lines),
        "rust.inline-tests" => Some(file.tests.len()),
        _ => None,
    }
}

const fn locked_kind(kind: DependencyKind) -> LockedDependencyKind {
    match kind {
        DependencyKind::Normal => LockedDependencyKind::Normal,
        DependencyKind::Development => LockedDependencyKind::Development,
        DependencyKind::Build => LockedDependencyKind::Build,
    }
}

pub(super) fn read_optional_lock(root: &Path, path: &Path) -> Result<Option<LockFile>, CheckError> {
    let path = resolve(root, path)?;
    LockFile::read_optional(&path).map_err(|error| CheckError::from_message(error.to_string()))
}
