//! Lock identity for every input governing one exact expansion manifest.

mod definition;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::LockedItemMacroManifest;

use crate::{
    cargo::ResolvedCargoGraph,
    source::{CompilationDomain, SourceIndex, SyntaxGuard},
};

use super::{AppliedItemMacroManifest, CheckError};
use definition::resolved_definition;

pub(crate) fn locked(
    applied: Vec<AppliedItemMacroManifest>,
    contract: &zrail_core::Contract,
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
    repository_implementations: &[zrail_core::LockedMacroImplementation],
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
) -> Result<Vec<LockedItemMacroManifest>, CheckError> {
    let mut locked = Vec::new();
    for applied in applied {
        let allowance = &contract.source.rust.item_macros[applied.allowance];
        let file = source
            .files
            .iter()
            .find(|file| file.relative == applied.invocation_path)
            .ok_or_else(|| manifest_error(&applied, "invocation source is unavailable"))?;
        let expansion = file
            .macro_expansions
            .iter()
            .find(|expansion| expansion.span == applied.invocation_span)
            .ok_or_else(|| manifest_error(&applied, "invocation expansion is unavailable"))?;
        let (definition, definition_sha256) = resolved_definition(
            allowance,
            expansion,
            resolved_cargo,
            repository_implementations,
        )
        .map_err(|message| manifest_error(&applied, &message))?;
        let guard = &expansion.observation.guard;
        let domains = active_domains(&applied.invocation_path, guard, compilation_domains);
        if domains.is_empty() {
            return Err(manifest_error(
                &applied,
                "invocation is unavailable in every Cargo compilation domain",
            ));
        }
        locked.push(LockedItemMacroManifest {
            name: allowance.name.clone(),
            invocation_path: applied.invocation_path,
            manifest_path: applied.manifest_path,
            manifest_sha256: applied.manifest_sha256,
            invocation_sha256: applied.invocation_sha256,
            definition,
            definition_sha256,
            guard: guard.canonical_name(),
            domains,
            bindings: applied.bindings,
        });
    }
    locked.sort();
    Ok(locked)
}

fn active_domains(
    path: &str,
    guard: &SyntaxGuard,
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
) -> Vec<String> {
    compilation_domains
        .get(path)
        .into_iter()
        .flatten()
        .filter(|domain| guard.availability_in_domain(domain).is_available())
        .map(CompilationDomain::canonical_identity)
        .collect()
}

fn manifest_error(applied: &AppliedItemMacroManifest, message: &str) -> CheckError {
    CheckError::from_message(format!(
        "exact item-macro manifest {:?} at {}: {message}",
        applied.manifest_path, applied.invocation_path
    ))
}
