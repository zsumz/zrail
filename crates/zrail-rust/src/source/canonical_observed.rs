//! Canonicalization considers every candidate without multiplying invocation results.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisLimits;

use crate::cargo::CargoWorkspace;

use super::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot,
    ResolvedModuleEdge, RustFileFacts,
};

#[derive(Clone, Copy)]
pub(crate) struct CanonicalizationContext<'a> {
    pub(crate) cargo: &'a CargoWorkspace,
    pub(crate) packages: &'a BTreeMap<String, BTreeSet<String>>,
    pub(crate) module_edges: &'a [ResolvedModuleEdge],
    pub(crate) compilation_domains: &'a BTreeMap<String, BTreeSet<CompilationDomain>>,
    pub(crate) compilation_roots: &'a [CompilationRoot],
    pub(crate) compilation_edges: &'a [CompilationModuleEdge],
    pub(crate) compilation_includes: &'a [CompilationIncludeEdge],
    pub(crate) analysis_limits: &'a AnalysisLimits,
}

pub(super) fn roots(file: &RustFileFacts) -> BTreeSet<String> {
    file.paths
        .iter()
        .chain(&file.calls)
        .chain(file.operations.iter().map(|operation| &operation.identity))
        .chain(&file.macros)
        .chain(file.macro_expansions.iter().flat_map(|fact| {
            fact.candidates
                .iter()
                .map(|candidate| &candidate.observation)
        }))
        .chain(file.opaque_macro_inputs.iter().flat_map(|fact| {
            fact.candidates
                .iter()
                .map(|candidate| &candidate.observation)
        }))
        .chain(&file.item_macros)
        .chain(file.compile_effects.iter().flat_map(|effect| {
            effect
                .invocation
                .candidates
                .iter()
                .map(|candidate| &candidate.observation)
        }))
        .filter_map(|fact| split_root(&fact.name).map(|root| visible_root(root).into()))
        .collect()
}

fn split_root(path: &str) -> Option<&str> {
    (!path.is_empty()).then(|| path.split("::").next().unwrap_or(path))
}

fn visible_root(root: &str) -> &str {
    root.strip_prefix("r#").unwrap_or(root)
}
