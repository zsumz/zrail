//! Evaluation order over the shared repository fact model.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Contract, DiagnosticLimit, FindingSink, LockFile};

use crate::{
    cargo::{CargoWorkspace, ResolvedCargoGraph, ResolvedFeatureWorld},
    inventory::RepositoryInventory,
    source::{CompilationDomain, ResolvedModuleEdge, SourceIndex},
};

use super::{
    capability, cargo_identity, cargo_override, dependency, evidence, file_role, generated,
    hygiene, macro_expansion, repository, size, source_shape, test_placement, type_policy,
};

pub(crate) struct RuleContext<'a> {
    pub(crate) contract: &'a Contract,
    pub(crate) lock: Option<&'a LockFile>,
    pub(crate) inventory: &'a RepositoryInventory,
    pub(crate) cargo: &'a CargoWorkspace,
    pub(crate) resolved_cargo: Option<&'a ResolvedCargoGraph>,
    pub(crate) source: &'a SourceIndex,
    pub(crate) module_edges: &'a [ResolvedModuleEdge],
    pub(crate) compilation_domains: &'a BTreeMap<String, BTreeSet<CompilationDomain>>,
    pub(crate) feature_worlds: &'a [ResolvedFeatureWorld],
}

pub(crate) fn evaluate(context: &RuleContext<'_>, limit: DiagnosticLimit) -> FindingSink {
    let mut findings =
        FindingSink::from_findings_with_limit(context.source.findings.clone(), limit);
    repository::evaluate(context, &mut findings);
    cargo_override::evaluate(context, &mut findings);
    cargo_identity::evaluate(context, &mut findings);
    generated::evaluate(context, &mut findings);
    dependency::evaluate(context, &mut findings);
    capability::evaluate(context, &mut findings);
    macro_expansion::evaluate(context, &mut findings);
    type_policy::evaluate(context, &mut findings);
    file_role::evaluate(context, &mut findings);
    source_shape::evaluate(context, &mut findings);
    hygiene::evaluate(context, &mut findings);
    test_placement::evaluate(context, &mut findings);
    evidence::evaluate(context, &mut findings);
    size::evaluate(context, &mut findings);
    findings
}
