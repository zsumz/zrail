//! Cargo dependency roots canonicalize policy paths without hiding source spelling.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, Finding};

use crate::cargo::{CrateRootAuthority, Package, rust_crate_root};

use super::{
    CanonicalizationContext, ObservedFact, SourceIndex, macro_definitions::MacroDefinitions,
};

const MAX_IDENTITIES_PER_ROOT: usize = 4;

pub(crate) fn canonicalize(
    index: &mut SourceIndex,
    context: CanonicalizationContext<'_>,
    review_bindings: impl FnOnce(&SourceIndex) -> super::BindingMacroPolicy,
) {
    let CanonicalizationContext {
        cargo,
        packages: contexts,
        module_edges,
        compilation_domains,
        compilation_roots,
        compilation_edges,
        compilation_includes,
    } = context;
    let packages = cargo
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let macro_definitions = MacroDefinitions::collect(
        index,
        cargo,
        compilation_domains,
        compilation_roots,
        compilation_edges,
        compilation_includes,
    );
    let macro_visibility = super::macro_visibility::MacroVisibility::collect(index, module_edges);
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
            let local_macros = macro_definitions.local_names(&file.relative, expansion.guard);
            macro_visibility.resolve(
                expansion,
                &file.relative,
                file.reachability,
                local_macros.as_ref(),
            );
            macro_definitions.apply(&file.relative, expansion);
        }
        let observed = super::canonical_observed::roots(file);
        let (roots, overflowed) = dependency_roots(&selected, &observed);
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
    let binding_macros = review_bindings(index);
    binding_macros.apply(index);
    let include_bindings = super::include_bindings::IncludeBindings::collect(
        index,
        compilation_roots,
        compilation_edges,
        compilation_includes,
        &binding_macros,
    );
    findings.extend(include_bindings.apply(index));
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
