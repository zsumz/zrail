//! Deterministic package input capture and digesting.

mod packages;

use std::{collections::BTreeMap, path::Path};

use zrail_core::{LockedMacroImplementation, MAX_INPUT_BYTES, read_bytes_with_limit, sha256_hex};

use crate::{
    cargo::CargoWorkspace,
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{SourceIndex, join_relative, parent},
};

use super::CheckError;

pub(super) const MAX_IMPLEMENTATION_INPUTS: usize = 4_096;
const MAX_IMPLEMENTATION_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn repository_manifest(
    inventory: &RepositoryInventory,
    cargo: &CargoWorkspace,
    source: &SourceIndex,
    package_name: &str,
    directory: &str,
) -> Result<LockedMacroImplementation, CheckError> {
    let package = cargo
        .packages
        .iter()
        .find(|package| package.name == package_name && package.directory == directory);
    let entries = inventory
        .entries
        .iter()
        .filter(|entry| entry.kind == RepositoryEntryKind::File)
        .map(|entry| (entry.relative.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut inputs = BTreeMap::<String, Vec<u8>>::new();
    let manifest_path = package.map_or_else(
        || repository_path(directory, "Cargo.toml"),
        crate::cargo::Package::manifest_path,
    );
    add_required_entry(&entries, &manifest_path, &mut inputs)?;
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
    let digest = digest_inputs(&inputs)?;
    Ok(LockedMacroImplementation {
        package: package_name.into(),
        directory: directory.into(),
        manifest_sha256: digest,
    })
}

fn add_package_inputs(
    cargo: &CargoWorkspace,
    source: &SourceIndex,
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    package: &crate::cargo::Package,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    for path in entries.keys().copied().filter(|path| {
        is_rust_source(path) && package_owns_path(&package.directory, path, &cargo.packages)
    }) {
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

fn is_rust_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs") || extension == "rsi")
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
        let Some(target) = effect.target.as_deref() else {
            continue;
        };
        let Ok(path) = join_relative(&parent(&file.relative), target) else {
            continue;
        };
        add_entry(entries, &path, inputs)?;
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
            "repository macro implementation manifest {path:?} is unavailable"
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
    let bytes = read_bytes_with_limit(&entry.absolute, MAX_INPUT_BYTES)
        .map_err(CheckError::from_message)?;
    inputs.insert(path.into(), bytes);
    Ok(())
}

pub(super) fn digest_inputs(inputs: &BTreeMap<String, Vec<u8>>) -> Result<String, CheckError> {
    if inputs.len() > MAX_IMPLEMENTATION_INPUTS {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_INPUTS}-input safety limit"
        )));
    }
    let total = inputs.iter().try_fold(0_usize, |total, (path, bytes)| {
        total
            .checked_add(path.len())
            .and_then(|value| value.checked_add(bytes.len()))
            .and_then(|value| value.checked_add(16))
    });
    if total.is_none_or(|total| total > MAX_IMPLEMENTATION_BYTES) {
        return Err(CheckError::from_message(format!(
            "macro implementation exceeds the {MAX_IMPLEMENTATION_BYTES}-byte safety limit"
        )));
    }
    let mut manifest = Vec::with_capacity(total.unwrap_or_default());
    for (path, bytes) in inputs {
        frame(&mut manifest, path.as_bytes());
        frame(&mut manifest, bytes);
    }
    Ok(sha256_hex(&manifest))
}

fn frame(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    output.extend_from_slice(bytes);
}
