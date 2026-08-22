//! Repository macro trust binds to a bounded deterministic package input manifest.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{LockedMacroImplementation, MAX_INPUT_BYTES, read_bytes_with_limit, sha256_hex};

use crate::{
    inventory::RepositoryEntryKind,
    source::{MacroOrigin, join_relative, parent},
};

use super::{CheckError, model::RepositoryModel};

const MAX_IMPLEMENTATION_INPUTS: usize = 4_096;
const MAX_IMPLEMENTATION_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn locked(
    model: &RepositoryModel,
) -> Result<Vec<LockedMacroImplementation>, CheckError> {
    let packages = trusted_packages(model);
    packages
        .into_iter()
        .map(|(package, directory)| manifest(model, &package, &directory))
        .collect()
}

fn trusted_packages(model: &RepositoryModel) -> BTreeSet<(String, String)> {
    let allowed = model
        .bundle
        .contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowance| allowance.name.as_str())
        .collect::<BTreeSet<_>>();
    model
        .source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
        .flat_map(|file| &file.macro_expansions)
        .filter(|expansion| expansion.names_covered_by(&allowed))
        .flat_map(crate::source::MacroExpansionFact::origins)
        .filter_map(|origin| match origin {
            MacroOrigin::Repository { package, directory } => {
                Some((package.clone(), directory.clone()))
            }
            _ => None,
        })
        .collect()
}

fn manifest(
    model: &RepositoryModel,
    package_name: &str,
    directory: &str,
) -> Result<LockedMacroImplementation, CheckError> {
    let package = model
        .cargo
        .packages
        .iter()
        .find(|package| package.name == package_name && package.directory == directory);
    let entries = model
        .inventory
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
        for file in model.source.files.iter().filter(|file| {
            !file.reachability.is_unreachable() && file.packages.contains(&package.name)
        }) {
            inputs.insert(file.relative.clone(), file_source(model, &file.relative)?);
            add_compile_inputs(&entries, file, &mut inputs)?;
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

fn add_compile_inputs(
    entries: &BTreeMap<&str, &crate::inventory::RepositoryEntry>,
    file: &crate::source::RustFileFacts,
    inputs: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), CheckError> {
    for effect in &file.compile_effects {
        if !effect.invocation.is_compiler_builtin() {
            continue;
        }
        if !matches!(
            effect.invocation.name.as_str(),
            "include" | "include_str" | "include_bytes"
        ) {
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

fn file_source(model: &RepositoryModel, path: &str) -> Result<Vec<u8>, CheckError> {
    model
        .inventory
        .rust_files
        .iter()
        .find(|file| file.relative == path)
        .map(|file| file.source.as_bytes().to_vec())
        .ok_or_else(|| {
            CheckError::from_message(format!("Rust implementation input {path:?} is unavailable"))
        })
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

fn digest_inputs(inputs: &BTreeMap<String, Vec<u8>>) -> Result<String, CheckError> {
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

#[cfg(test)]
#[path = "macro_implementations_test.rs"]
mod macro_implementations_test;
