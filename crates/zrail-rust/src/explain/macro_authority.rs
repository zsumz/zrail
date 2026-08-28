//! Path guidance names every observed content-bound repository macro package.

use std::collections::BTreeSet;

use crate::{cargo::DependencySource, engine::RepositoryModel, source::MacroOrigin};

use super::{ItemMacroAuthorityExplanation, MacroInvocationExplanation};
use zrail_core::MacroBindingMode;

pub(super) fn item_authorities(
    model: &RepositoryModel,
    path: &str,
) -> Vec<ItemMacroAuthorityExplanation> {
    model
        .source
        .files
        .iter()
        .filter(|file| file.relative == path)
        .flat_map(|file| {
            crate::rules::source_graph::item_macro_authorities(
                &model.bundle.contract,
                file,
                model.resolved_cargo.as_ref(),
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|index| {
            let allowance = &model.bundle.contract.source.rust.item_macros[index];
            ItemMacroAuthorityExplanation {
                name: allowance.name.clone(),
                selector: crate::rules::source_graph::item_macro_selector(allowance),
                binding: allowance.binding.map_or_else(
                    || "name-only".into(),
                    |binding| {
                        match binding {
                            MacroBindingMode::Exact => "exact",
                            MacroBindingMode::Conservative => "conservative",
                        }
                        .into()
                    },
                ),
                source: allowance
                    .source
                    .as_ref()
                    .map(zrail_core::CrateRootSource::identity),
                reason: allowance.reason.clone(),
            }
        })
        .collect()
}

pub(super) fn implementations(model: &RepositoryModel) -> Vec<String> {
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
        .flat_map(|file| &file.macro_expansions)
        .flat_map(|expansion| {
            expansion.candidates.iter().flat_map(|candidate| {
                candidate
                    .allowance_names(&expansion.name)
                    .into_iter()
                    .filter(|name| allowed.contains(name))
                    .flat_map(|name| {
                        candidate
                            .origins
                            .iter()
                            .filter_map(move |origin| match origin {
                                MacroOrigin::Repository { package, directory } => {
                                    Some(format!("{name}@{package}:{directory}"))
                                }
                                _ => None,
                            })
                    })
            })
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn invocations(model: &RepositoryModel, path: &str) -> Vec<MacroInvocationExplanation> {
    model
        .source
        .files
        .iter()
        .filter(|file| file.relative == path)
        .flat_map(|file| &file.macro_expansions)
        .map(|expansion| {
            let origins = expansion
                .origins()
                .map(origin_name)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            MacroInvocationExplanation {
                written: expansion.name.clone(),
                preferred: expansion.preferred_policy_name().map(str::to_owned),
                origins,
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn origin_name(origin: &MacroOrigin) -> String {
    match origin {
        MacroOrigin::Pending { local_module: true } => "pending:repository".into(),
        MacroOrigin::Pending {
            local_module: false,
        } => "pending".into(),
        MacroOrigin::CompilerBuiltin => "compiler".into(),
        MacroOrigin::Repository { package, directory } => {
            format!("repository:{package}:{directory}")
        }
        MacroOrigin::External { package, source } => {
            format!("external:{package}:{}", dependency_source(source))
        }
        MacroOrigin::Unresolved => "unresolved".into(),
    }
}

fn dependency_source(source: &DependencySource) -> String {
    match source {
        DependencySource::WorkspaceMember { directory, .. } => format!("workspace:{directory}"),
        DependencySource::RepositoryPath { path, .. } => format!("path:{path}"),
        DependencySource::Registry {
            registry,
            index,
            requirement,
        } => format!(
            "registry:{}:{requirement}",
            registry
                .as_deref()
                .or(index.as_deref())
                .unwrap_or("crates.io")
        ),
        DependencySource::Git {
            repository,
            branch,
            tag,
            rev,
            ..
        } => format!(
            "git:{repository}:{}",
            rev.as_deref()
                .or(tag.as_deref())
                .or(branch.as_deref())
                .unwrap_or("HEAD")
        ),
    }
}
