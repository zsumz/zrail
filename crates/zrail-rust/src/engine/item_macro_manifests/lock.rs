//! Lock identity for every input governing one exact expansion manifest.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, LockedItemMacroManifest};

use crate::{
    cargo::{DependencySource, ResolvedCargoGraph, ResolvedPackageIdentity, source_matches},
    source::{CompilationDomain, MacroExpansionFact, MacroOrigin, SourceIndex, SyntaxGuard},
};

use super::{AppliedItemMacroManifest, CheckError};

pub(crate) fn locked(
    applied: Vec<AppliedItemMacroManifest>,
    contract: &zrail_core::Contract,
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
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
        let (definition, definition_sha256) =
            resolved_definition(allowance, expansion, resolved_cargo)
                .map_err(|message| manifest_error(&applied, &message))?;
        let guard = expansion.observation.guard;
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
            guard: guard.canonical_name().into(),
            domains,
            bindings: applied.bindings,
        });
    }
    locked.sort();
    Ok(locked)
}

fn resolved_definition(
    allowance: &zrail_core::ItemMacroContract,
    expansion: &MacroExpansionFact,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Result<(String, String), String> {
    if expansion.observation.quality != AnalysisQuality::Exact || expansion.candidates.is_empty() {
        return Err("macro definition resolution is not exact".into());
    }
    let mut definitions = BTreeMap::<String, String>::new();
    for candidate in &expansion.candidates {
        if candidate.observation.quality != AnalysisQuality::Exact
            || !candidate
                .allowance_names(&expansion.observation.name)
                .contains(&allowance.name.as_str())
        {
            return Err("macro definition has an uncovered or inexact candidate".into());
        }
        if candidate.origins.is_empty() {
            return Err("macro definition has no proven origin".into());
        }
        for origin in &candidate.origins {
            let (identity, sha256) = match origin {
                MacroOrigin::Repository { .. } => local_definition(allowance, candidate)?,
                MacroOrigin::External { package, source } => {
                    external_definition(allowance, package, source, resolved_cargo)?
                }
                MacroOrigin::CompilerBuiltin => {
                    return Err("compiler-owned item macros cannot use expansion manifests".into());
                }
                MacroOrigin::Pending { .. } | MacroOrigin::Unresolved => {
                    return Err("macro definition origin is unresolved".into());
                }
            };
            if definitions
                .insert(identity.clone(), sha256.clone())
                .is_some_and(|current| current != sha256)
            {
                return Err(format!(
                    "macro definition {identity:?} resolved to conflicting content digests"
                ));
            }
        }
    }
    match definitions.into_iter().collect::<Vec<_>>().as_slice() {
        [definition] => Ok(definition.clone()),
        [] => Err("macro definition has no content-bound identity".into()),
        definitions => Err(format!(
            "macro definition is ambiguous across {} content-bound identities",
            definitions.len()
        )),
    }
}

fn local_definition(
    allowance: &zrail_core::ItemMacroContract,
    candidate: &crate::source::MacroCandidate,
) -> Result<(String, String), String> {
    if allowance.source.is_some() {
        return Err("repository macro definition does not match external source authority".into());
    }
    let path = candidate
        .definition
        .as_deref()
        .ok_or_else(|| "repository macro definition path is unresolved".to_owned())?;
    let name = candidate
        .definition_name
        .as_deref()
        .ok_or_else(|| "repository macro definition name is unresolved".to_owned())?;
    let sha256 = candidate
        .definition_sha256
        .as_deref()
        .ok_or_else(|| "repository macro definition content is unresolved".to_owned())?;
    Ok((format!("repository:{path}::{name}"), sha256.into()))
}

fn external_definition(
    allowance: &zrail_core::ItemMacroContract,
    package: &str,
    observed: &DependencySource,
    resolved_cargo: Option<&ResolvedCargoGraph>,
) -> Result<(String, String), String> {
    let allowed = allowance.source.as_ref().ok_or_else(|| {
        "external macro definition requires explicit immutable source authority".to_owned()
    })?;
    let graph =
        resolved_cargo.ok_or_else(|| "external macro definition requires Cargo.lock".to_owned())?;
    let identity = match allowed {
        zrail_core::CrateRootSource::CargoLock {
            package: selected,
            version,
            source,
        } => {
            let selected = graph.lookup(selected, version.as_deref(), source.as_deref())?;
            let actual = graph.package_for_source(package, observed)?;
            if selected != actual {
                return Err("external macro definition does not match Cargo.lock authority".into());
            }
            actual
        }
        _ if source_matches(allowed, observed) => graph.package_for_source(package, observed)?,
        _ => return Err("external macro definition does not match source authority".into()),
    };
    let checksum = identity.checksum.as_ref().ok_or_else(|| {
        format!(
            "exact external item-macro manifest requires a Cargo.lock package checksum: {}",
            identity.label()
        )
    })?;
    Ok((external_identity(identity), checksum.clone()))
}

fn external_identity(identity: &ResolvedPackageIdentity) -> String {
    format!(
        "cargo-lock:{}:{}:{}",
        identity.name, identity.version, identity.source
    )
}

fn active_domains(
    path: &str,
    guard: SyntaxGuard,
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
) -> Vec<String> {
    compilation_domains
        .get(path)
        .into_iter()
        .flatten()
        .filter(|domain| {
            guard.available_in(SyntaxGuard::for_test_only(domain.mode.enables_cfg_test()))
        })
        .map(CompilationDomain::canonical_identity)
        .collect()
}

fn manifest_error(applied: &AppliedItemMacroManifest, message: &str) -> CheckError {
    CheckError::from_message(format!(
        "exact item-macro manifest {:?} at {}: {message}",
        applied.manifest_path, applied.invocation_path
    ))
}
