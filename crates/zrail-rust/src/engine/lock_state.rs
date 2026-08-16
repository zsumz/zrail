//! Candidate lock generation and exact lock drift findings.

use std::{collections::BTreeMap, path::Path};

use zrail_core::{
    LockFile, LockedDependency, LockedDependencyKind, LockedDependencyScope,
    LockedDependencySource, LockedGate, LockedMacroDefinition, LockedPackage, LockedRatchet,
    input::{MAX_INPUT_BYTES, read_bytes_with_limit},
    sha256_hex,
};

use crate::{
    cargo::{DependencyKind, DependencySource},
    inventory::RepositoryEntryKind,
};

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
    lock.gates = locked_gates(model)?;
    lock.macros = locked_macros(model);
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

fn locked_macros(model: &RepositoryModel) -> Vec<LockedMacroDefinition> {
    let files = model
        .source
        .files
        .iter()
        .map(|file| (file.relative.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut locked = Vec::new();
    for allowance in &model.bundle.contract.source.rust.macros.allow {
        let Some(path) = allowance.definition.as_deref() else {
            continue;
        };
        let Some(file) = files.get(path) else {
            continue;
        };
        let name = allowance
            .name
            .rsplit("::")
            .next()
            .unwrap_or(&allowance.name);
        for (ordinal, definition) in file
            .macro_definitions
            .iter()
            .filter(|definition| definition.name == name)
            .enumerate()
        {
            locked.push(LockedMacroDefinition {
                path: path.into(),
                name: allowance.name.clone(),
                ordinal: ordinal + 1,
                sha256: definition.sha256.clone(),
            });
        }
    }
    locked
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

fn locked_gates(model: &RepositoryModel) -> Result<Vec<LockedGate>, CheckError> {
    let entries = model
        .inventory
        .entries
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut gates = Vec::new();
    for gate in &model.bundle.contract.gates {
        let Some(entry) = entries.get(gate.path.as_str()) else {
            continue;
        };
        if entry.kind != RepositoryEntryKind::File {
            continue;
        }
        let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)
            .map_err(CheckError::from_message)?;
        gates.push(LockedGate {
            name: gate.name.clone(),
            path: gate.path.clone(),
            sha256: sha256_hex(&bytes),
        });
    }
    Ok(gates)
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
