//! The complete, exact result of source-graph traversal.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::Finding;

use crate::source::{
    CompilationDomain, CompilationIncludeEdge, CompilationModuleEdge, CompilationRoot,
    Reachability, ResolvedModuleEdge,
};

pub(crate) struct SourceGraphAnalysis {
    pub(crate) reachability: BTreeMap<String, Reachability>,
    pub(crate) packages: BTreeMap<String, BTreeSet<String>>,
    pub(crate) compilation_domains: BTreeMap<String, BTreeSet<CompilationDomain>>,
    pub(crate) compilation_roots: Vec<CompilationRoot>,
    pub(crate) compilation_edges: Vec<CompilationModuleEdge>,
    pub(crate) compilation_includes: Vec<CompilationIncludeEdge>,
    pub(crate) module_edges: Vec<ResolvedModuleEdge>,
    pub(crate) findings: Vec<Finding>,
}
