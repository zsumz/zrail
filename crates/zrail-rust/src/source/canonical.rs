//! Cargo dependency roots canonicalize policy paths without hiding source spelling.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding};

use crate::cargo::{CargoWorkspace, CrateRootAuthority, Package, rust_crate_root};

use super::{
    ObservedFact, SourceIndex,
    macro_definitions::{local_macro_names, package_macro_definitions},
};

const MAX_IDENTITIES_PER_ROOT: usize = 4;
#[cfg(test)]
use super::macro_definitions::{MAX_MACRO_DEFINITIONS_PER_PACKAGE, MacroDefinitionSet};

pub(crate) fn canonicalize(
    index: &mut SourceIndex,
    cargo: &CargoWorkspace,
    contexts: &BTreeMap<String, BTreeSet<String>>,
) {
    let packages = cargo
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let macro_definitions = package_macro_definitions(index, contexts);
    let macro_visibility = super::macro_visibility::MacroVisibility::collect(index);
    let mut findings = Vec::new();
    for file in &mut index.files {
        let selected: Vec<&Package> = contexts.get(&file.relative).map_or_else(
            || {
                package_for_file(&cargo.packages, &file.relative)
                    .into_iter()
                    .collect()
            },
            |names| {
                names
                    .iter()
                    .filter_map(|name| packages.get(name.as_str()).copied())
                    .collect()
            },
        );
        let local_macros = local_macro_names(&selected, &macro_definitions);
        for expansion in &mut file.macros {
            if !expansion.name.contains("::")
                && local_macros
                    .as_ref()
                    .is_none_or(|names| names.contains(expansion.name.as_str()))
            {
                expansion.canonical.clear();
                expansion.quality = AnalysisQuality::Unresolved;
            }
        }
        for expansion in file
            .macro_expansions
            .iter_mut()
            .chain(&mut file.opaque_macro_inputs)
            .chain(
                file.compile_effects
                    .iter_mut()
                    .map(|effect| &mut effect.invocation),
            )
        {
            for candidate in &mut expansion.candidates {
                let observed = &mut candidate.observation;
                if !observed.name.contains("::")
                    && local_macros
                        .as_ref()
                        .is_none_or(|names| names.contains(observed.name.as_str()))
                {
                    observed.canonical.clear();
                    observed.quality = AnalysisQuality::Unresolved;
                }
            }
            expansion.refresh_quality();
            macro_visibility.resolve(expansion, &file.relative, local_macros.as_ref());
        }
        let observed = super::canonical_observed::roots(file);
        let (roots, overflowed) = dependency_roots(&selected, &observed);
        findings.extend(
            overflowed
                .iter()
                .map(|root| identity_limit(&file.relative, root)),
        );
        for fact in file
            .paths
            .iter_mut()
            .chain(&mut file.calls)
            .chain(&mut file.macros)
            .chain(&mut file.item_macros)
        {
            canonicalize_fact_bounded(fact, &roots, &overflowed);
        }
        for expansion in file
            .macro_expansions
            .iter_mut()
            .chain(&mut file.opaque_macro_inputs)
            .chain(
                file.compile_effects
                    .iter_mut()
                    .map(|effect| &mut effect.invocation),
            )
        {
            for candidate in &mut expansion.candidates {
                canonicalize_fact_bounded(&mut candidate.observation, &roots, &overflowed);
            }
            super::macro_origins::resolve(expansion, &selected);
        }
    }
    index.findings.extend(findings);
}

fn dependency_roots(
    packages: &[&Package],
    observed: &BTreeSet<String>,
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>) {
    let mut roots = BTreeMap::<String, BTreeSet<String>>::new();
    let mut overflowed = BTreeSet::new();
    for package in packages {
        for dependency in &package.dependencies {
            if dependency.crate_root_authority == CrateRootAuthority::Unresolved {
                continue;
            }
            let crate_root = rust_crate_root(&dependency.crate_root);
            if !observed.contains(&crate_root) || overflowed.contains(&crate_root) {
                continue;
            }
            let canonical = rust_crate_root(&dependency.name);
            let identities = roots.entry(crate_root.clone()).or_default();
            if identities.len() == MAX_IDENTITIES_PER_ROOT && !identities.contains(&canonical) {
                overflowed.insert(crate_root);
            } else {
                identities.insert(canonical);
            }
        }
    }
    (roots, overflowed)
}

fn package_for_file<'a>(packages: &'a [Package], file: &str) -> Option<&'a Package> {
    packages
        .iter()
        .filter(|package| package.contains_file(file))
        .max_by_key(|package| package.directory.len())
}

fn canonicalize_fact(fact: &mut ObservedFact, roots: &BTreeMap<String, BTreeSet<String>>) {
    // Exact lexical module bindings already own their policy identity.
    if !fact.canonical.is_empty() {
        return;
    }
    let Some((root, suffix)) = split_root(&fact.name) else {
        return;
    };
    let visible_root = visible_root(root);
    let Some(canonical_roots) = roots.get(visible_root) else {
        return;
    };
    let canonical = canonical_roots
        .iter()
        .map(|canonical| format!("{canonical}{suffix}"))
        .collect::<Vec<_>>();
    if canonical.len() != 1 || canonical[0] != fact.name {
        fact.canonical = canonical;
    }
    if canonical_roots.len() > 1 && fact.quality == AnalysisQuality::Exact {
        fact.quality = AnalysisQuality::Conservative;
    }
}

fn canonicalize_fact_bounded(
    fact: &mut ObservedFact,
    roots: &BTreeMap<String, BTreeSet<String>>,
    overflowed: &BTreeSet<String>,
) {
    if split_root(&fact.name).is_some_and(|(root, _)| overflowed.contains(visible_root(root))) {
        fact.canonical.clear();
        fact.quality = AnalysisQuality::Unresolved;
    } else {
        canonicalize_fact(fact, roots);
    }
}

fn visible_root(root: &str) -> &str {
    root.strip_prefix("r#").unwrap_or(root)
}

fn identity_limit(path: &str, root: &str) -> Finding {
    Finding::error(
        "RUST-CANON-001",
        "rust.source.dependency-identity",
        "source",
        format!(
            "Cargo dependency root {root:?} exceeds the {MAX_IDENTITIES_PER_ROOT}-identity analysis limit"
        ),
    )
    .at(path, None)
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help("split shared source or use distinct dependency aliases so policy identity is exact")
}

fn split_root(path: &str) -> Option<(&str, &str)> {
    if path.is_empty() {
        return None;
    }
    Some(path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    }))
}

#[cfg(test)]
#[path = "canonical_test.rs"]
mod canonical_test;
