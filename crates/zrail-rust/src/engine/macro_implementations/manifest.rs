//! Deterministic package input capture and digesting.

mod digest;
mod packages;
mod resolution;
mod scan;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{LockedMacroImplementation, MAX_INPUT_BYTES, read_bytes_with_limit};

use crate::{
    cargo::{CargoWorkspace, ResolvedCargoGraph},
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
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Result<LockedMacroImplementation, CheckError> {
    let package = cargo
        .packages
        .iter()
        .find(|package| package.name == package_name && package.directory == directory)
        .ok_or_else(|| CheckError::from_message(format!(
            "repository macro provider {package_name:?} at {directory:?} has no active Cargo manifest"
        )))?;
    let packages = packages::implementation_packages(&cargo.packages, package)?;
    let mut compile_inputs = BTreeSet::new();
    for package in &packages {
        for file in source.files.iter().filter(|file| {
            !file.reachability.is_unreachable() && file.packages.contains(&package.name)
        }) {
            collect_compile_inputs(file, &mut compile_inputs)?;
        }
    }
    let captured = scan::inputs(
        &inventory.root,
        cargo,
        &packages,
        extra_inputs,
        &compile_inputs,
    )?;
    let entries = captured
        .iter()
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut inputs = BTreeMap::<String, Vec<u8>>::new();
    for path in entries.keys() {
        add_entry(&entries, path, &mut inputs)?;
    }
    for package in &packages {
        add_required_entry(&entries, &package.manifest_path(), &mut inputs)?;
    }
    for path in compile_inputs {
        add_required_entry(&entries, &path, &mut inputs)?;
    }
    resolution::validate(
        &packages,
        resolved_cargo,
        inputs.get("Cargo.lock").map(Vec::as_slice),
    )?;
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

pub(super) fn reserved(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, ".git" | ".zrail" | "target" | "zrail.lock"))
}

fn collect_compile_inputs(
    file: &crate::source::RustFileFacts,
    inputs: &mut BTreeSet<String>,
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
        inputs.insert(path);
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
