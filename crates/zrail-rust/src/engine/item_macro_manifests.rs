//! Reviewed item-macro manifests contribute exact lexical namespace bindings.

mod lock;

use std::collections::BTreeSet;

use zrail_core::{
    AnalysisQuality, ItemMacroBindingKind, ItemMacroManifest, MAX_INPUT_BYTES,
    read_text_with_limit, sha256_hex,
};

use crate::{
    inventory::{RepositoryEntryKind, RepositoryInventory},
    source::{BindingAnchor, BindingKind, BindingVisibility, ImportBindingFact, SourceIndex},
};

use super::CheckError;

pub(super) use lock::locked;

const MAX_MANIFEST_BINDINGS: usize = 10_000;

pub(super) struct AppliedItemMacroManifest {
    allowance: usize,
    invocation_path: String,
    manifest_path: String,
    manifest_sha256: String,
    invocation_sha256: String,
    invocation_span: Option<zrail_core::SourceSpan>,
    bindings: usize,
}

pub(super) fn apply(
    inventory: &RepositoryInventory,
    contract: &zrail_core::Contract,
    source: &mut SourceIndex,
) -> Result<Vec<AppliedItemMacroManifest>, CheckError> {
    let mut applied = Vec::new();
    for (allowance_index, allowance) in contract
        .source
        .rust
        .item_macros
        .iter()
        .enumerate()
        .filter(|(_, allowance)| allowance.manifest.is_some())
    {
        let path = allowance.path.as_deref().ok_or_else(|| {
            CheckError::from_message("exact item-macro manifests require one invocation path")
        })?;
        let Some(manifest_path) = allowance.manifest.as_deref() else {
            continue;
        };
        let text = manifest_text(inventory, manifest_path)?;
        let manifest = parse_manifest(&text, &allowance.name)?;
        let file = source
            .files
            .iter_mut()
            .find(|file| file.relative == path)
            .ok_or_else(|| {
                CheckError::from_message(format!(
                    "item-macro manifest invocation source is unavailable: {path}"
                ))
            })?;
        let (guard, lexical_scope, invocation_sha256, invocation_span) = {
            let mut matches = file
                .item_macros
                .iter()
                .filter(|fact| fact.policy_names().any(|name| name == allowance.name))
                .filter_map(|fact| {
                    file.macro_expansions
                        .iter()
                        .find(|expansion| expansion.span == fact.span)
                        .map(|expansion| (fact, expansion))
                });
            let (fact, expansion) = matches.next().ok_or_else(|| {
                CheckError::from_message(format!(
                    "exact item-macro manifest matches no invocation: {} at {path}",
                    allowance.name
                ))
            })?;
            if matches.next().is_some() {
                return Err(CheckError::from_message(format!(
                    "exact item-macro manifest is ambiguous across multiple invocations: {} at {path}",
                    allowance.name
                )));
            }
            if manifest.invocation_sha256 != expansion.input_sha256 {
                return Err(CheckError::from_message(format!(
                    "item-macro invocation tokens differ from exact manifest {manifest_path:?}"
                )));
            }
            (
                fact.guard.clone(),
                fact.lexical_scope.clone(),
                expansion.input_sha256.clone(),
                fact.span,
            )
        };
        for binding in &manifest.bindings {
            file.import_bindings.push(ImportBindingFact {
                name: Some(binding.name.clone()),
                target: binding.name.clone(),
                kind: binding_kind(binding.kind),
                anchor: BindingAnchor::Lexical,
                visibility: if binding.public {
                    BindingVisibility::Public
                } else {
                    BindingVisibility::Private
                },
                quality: AnalysisQuality::Exact,
                quality_without_macros: AnalysisQuality::Exact,
                replacement_macros: Vec::new(),
                guard: guard.clone(),
                lexical_scope: lexical_scope.clone(),
            });
        }
        applied.push(AppliedItemMacroManifest {
            allowance: allowance_index,
            invocation_path: path.into(),
            manifest_path: manifest_path.into(),
            manifest_sha256: sha256_hex(text.as_bytes()),
            invocation_sha256,
            invocation_span,
            bindings: manifest.bindings.len(),
        });
    }
    Ok(applied)
}

fn manifest_text(inventory: &RepositoryInventory, path: &str) -> Result<String, CheckError> {
    let entry = inventory
        .entries
        .iter()
        .find(|entry| entry.relative == path && entry.kind == RepositoryEntryKind::File)
        .ok_or_else(|| {
            CheckError::from_message(format!(
                "exact item-macro manifest is excluded or unavailable: {path}"
            ))
        })?;
    read_text_with_limit(&entry.absolute, MAX_INPUT_BYTES).map_err(CheckError::from_message)
}

fn parse_manifest(text: &str, name: &str) -> Result<ItemMacroManifest, CheckError> {
    let mut manifest = toml::from_str::<ItemMacroManifest>(text)
        .map_err(|error| CheckError::from_message(format!("parse item-macro manifest: {error}")))?;
    if manifest.schema != 1 || manifest.macro_name != name {
        return Err(CheckError::from_message(format!(
            "item-macro manifest must use schema 1 and macro_name {name:?}"
        )));
    }
    if manifest.bindings.is_empty() || manifest.bindings.len() > MAX_MANIFEST_BINDINGS {
        return Err(CheckError::from_message(format!(
            "item-macro manifest requires 1..={MAX_MANIFEST_BINDINGS} exact bindings"
        )));
    }
    manifest.bindings.sort();
    let mut names = BTreeSet::new();
    for binding in &manifest.bindings {
        let ident = syn::parse_str::<syn::Ident>(&binding.name).map_err(|_| {
            CheckError::from_message(format!(
                "item-macro manifest binding is not one Rust identifier: {:?}",
                binding.name
            ))
        })?;
        if ident != binding.name || !names.insert(binding.name.as_str()) {
            return Err(CheckError::from_message(format!(
                "item-macro manifest binding is noncanonical or duplicated: {:?}",
                binding.name
            )));
        }
    }
    Ok(manifest)
}

const fn binding_kind(kind: ItemMacroBindingKind) -> BindingKind {
    match kind {
        ItemMacroBindingKind::Type => BindingKind::LocalType,
        ItemMacroBindingKind::Constructor => BindingKind::LocalConstructor,
        ItemMacroBindingKind::Value => BindingKind::LocalValue,
    }
}
