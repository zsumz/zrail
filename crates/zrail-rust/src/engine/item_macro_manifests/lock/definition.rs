//! Content-bound item-macro definition identity resolution.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use crate::{
    cargo::{DependencySource, ResolvedCargoGraph, ResolvedPackageIdentity, source_matches},
    source::{MacroExpansionFact, MacroOrigin},
};

pub(super) fn resolved_definition(
    allowance: &zrail_core::ItemMacroContract,
    expansion: &MacroExpansionFact,
    resolved_cargo: Option<&ResolvedCargoGraph>,
    repository_implementations: &[zrail_core::LockedMacroImplementation],
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
                MacroOrigin::Repository { package, directory } => local_definition(
                    allowance,
                    candidate,
                    package,
                    directory,
                    repository_implementations,
                )?,
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
    package: &str,
    directory: &str,
    repository_implementations: &[zrail_core::LockedMacroImplementation],
) -> Result<(String, String), String> {
    match allowance.source.as_ref() {
        None => local_definition_tokens(candidate),
        Some(zrail_core::CrateRootSource::Repository {
            package: allowed_package,
            directory: allowed_directory,
        }) if allowed_package == package && allowed_directory == directory => {
            repository_implementations
                .iter()
                .find(|implementation| {
                    implementation.package == package && implementation.directory == directory
                })
                .map(|implementation| {
                    (
                        format!("repository:{package}:{directory}"),
                        implementation.manifest_sha256.clone(),
                    )
                })
                .ok_or_else(|| "repository macro implementation manifest is unavailable".to_owned())
        }
        Some(zrail_core::CrateRootSource::Repository { .. }) => {
            Err("repository macro definition does not match repository source authority".into())
        }
        Some(_) => {
            Err("repository macro definition does not match external source authority".into())
        }
    }
}

fn local_definition_tokens(
    candidate: &crate::source::MacroCandidate,
) -> Result<(String, String), String> {
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
