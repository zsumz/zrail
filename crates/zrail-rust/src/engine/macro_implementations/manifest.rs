//! Deterministic package input capture and digesting.

mod digest;
mod packages;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{LockedMacroImplementation, MAX_INPUT_BYTES, read_bytes_with_limit};

use crate::{
    cargo::CargoWorkspace,
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{SourceIndex, join_relative, parent},
};

use super::CheckError;

use digest::MAX_IMPLEMENTATION_BYTES;
pub(super) use digest::{MAX_IMPLEMENTATION_INPUTS, digest_inputs};

pub(super) fn repository_manifest(
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    source: &SourceIndex,
    package_name: &str,
    directory: &str,
    extra_inputs: &BTreeSet<String>,
) -> Result<LockedMacroImplementation, CheckError> {
    let package = cargo
        .packages
        .iter()
        .find(|package| package.name == package_name && package.directory == directory);
    let entries = inventory
        .entries
        .iter()
        .filter(|entry| entry.kind != RepositoryEntryKind::Directory && !reserved(&entry.relative))
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut inputs = BTreeMap::<String, Vec<u8>>::new();
    let manifest_path = package.map_or_else(
        || repository_path(directory, "Cargo.toml"),
        crate::cargo::Package::manifest_path,
    );
    add_required_entry(&entries, &manifest_path, &mut inputs)?;
    add_entry(&entries, "Cargo.lock", &mut inputs)?;
    if let Some(package) = package {
        add_entry(&entries, "Cargo.toml", &mut inputs)?;
        for package in packages::implementation_packages(&cargo.packages, package)? {
            add_required_entry(&entries, &package.manifest_path(), &mut inputs)?;
            add_package_inputs(cargo, source, &entries, package, &mut inputs)?;
        }
    } else {
        for path in entries
            .keys()
            .copied()
            .filter(|path| inside_directory(path, directory))
            .collect::<Vec<_>>()
        {
            add_entry(&entries, path, &mut inputs)?;
        }
    }
    for pattern in extra_inputs {
        let paths = entries
            .keys()
            .copied()
            .filter(|path| zrail_core::glob_matches(pattern, path))
            .collect::<Vec<_>>();
        if paths.is_empty() {
            return Err(CheckError::from_message(format!(
                "repository macro input {pattern:?} matches no bounded regular files"
            )));
        }
        for path in paths {
            add_required_entry(&entries, path, &mut inputs)?;
        }
    }
    let digest = digest_inputs(&inputs)?;
    Ok(LockedMacroImplementation {
        package: package_name.into(),
        directory: directory.into(),
        inputs_sha256: digest,
    })
}

fn add_package_inputs(
    cargo: &CargoWorkspace,
    source: &SourceIndex,
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    package: &crate::cargo::Package,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    for path in entries
        .keys()
        .copied()
        .filter(|path| package_owns_path(&package.directory, path, &cargo.packages))
    {
        add_entry(entries, path, inputs)?;
    }
    for file in source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable() && file.packages.contains(&package.name))
    {
        add_compile_inputs(entries, file, inputs)?;
    }
    Ok(())
}

fn reserved(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, ".git" | ".zrail" | "target" | "zrail.lock"))
}

fn package_owns_path(directory: &str, path: &str, packages: &[crate::cargo::Package]) -> bool {
    inside_directory(path, directory)
        && !packages.iter().any(|package| {
            package.directory != directory
                && package.directory != "."
                && inside_directory(path, &package.directory)
                && inside_directory(&package.directory, directory)
        })
}

fn add_compile_inputs(
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    file: &crate::source::RustFileFacts,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    for effect in &file.compile_effects {
        if !effect.invocation.is_compiler_builtin()
            || !matches!(
                effect.invocation.name.as_str(),
                "include" | "include_str" | "include_bytes"
            )
        {
            continue;
        }
        let target = effect.target.as_deref().ok_or_else(|| {
            CheckError::from_message(format!(
                "repository macro implementation {} has an unresolved include input",
                file.relative
            ))
        })?;
        let path = join_relative(&parent(&file.relative), target).map_err(|error| {
            CheckError::from_message(format!(
                "repository macro include input is invalid: {error:?}"
            ))
        })?;
        add_required_entry(entries, &path, inputs)?;
    }
    Ok(())
}

fn add_required_entry(
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    path: &str,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    if !entries.contains_key(path) {
        return Err(CheckError::from_message(format!(
            "repository macro implementation input {path:?} is unavailable"
        )));
    }
    add_entry(entries, path, inputs)
}

fn repository_path(directory: &str, name: &str) -> String {
    if directory == "." {
        name.into()
    } else {
        format!("{directory}/{name}")
    }
}

fn inside_directory(path: &str, directory: &str) -> bool {
    directory == "." || path == directory || path.starts_with(&format!("{directory}/"))
}

fn add_entry(
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    path: &str,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    if inputs.contains_key(path) {
        return Ok(());
    }
    let Some(entry) = entries.get(path) else {
        return Ok(());
    };
    if entry.kind != RepositoryEntryKind::File {
        return Err(CheckError::from_message(format!(
            "repository macro input {path:?} is not a regular file"
        )));
    }
    if inputs.len() == MAX_IMPLEMENTATION_INPUTS {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_INPUTS}-input safety limit"
        )));
    }
    let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)
        .map_err(CheckError::from_message)?;
    let captured = inputs
        .iter()
        .map(|(path, bytes)| path.len() + bytes.len() + 16)
        .sum::<usize>();
    if captured
        .saturating_add(path.len())
        .saturating_add(bytes.len())
        .saturating_add(16)
        > MAX_IMPLEMENTATION_BYTES
    {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_BYTES}-byte safety limit"
        )));
    }
    inputs.insert(path.into(), bytes);
    Ok(())
}
