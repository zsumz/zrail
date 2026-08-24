//! Canonicalization inputs keep Cargo and compilation authority together.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::CargoWorkspace;
use zrail_core::AnalysisLimits;

use super::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot,
    ResolvedModuleEdge,
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
