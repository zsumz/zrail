//! Evaluation order over the shared repository fact model.

use zrail_core::{Contract, FindingSink, LockFile};

use crate::{cargo::CargoWorkspace, inventory::RepositoryInventory, source::SourceIndex};

use super::{
    capability, cargo_override, dependency, evidence, generated, hygiene, repository, size,
    source_shape, test_placement,
};

pub(crate) struct RuleContext<'a> {
    pub(crate) contract: &'a Contract,
    pub(crate) lock: Option<&'a LockFile>,
    pub(crate) inventory: &'a RepositoryInventory,
    pub(crate) cargo: &'a CargoWorkspace,
    pub(crate) source: &'a SourceIndex,
}

pub(crate) fn evaluate(context: &RuleContext<'_>) -> FindingSink {
    let mut findings = FindingSink::from_findings(context.source.findings.clone());
    repository::evaluate(context, &mut findings);
    cargo_override::evaluate(context, &mut findings);
    generated::evaluate(context, &mut findings);
    dependency::evaluate(context, &mut findings);
    capability::evaluate(context, &mut findings);
    source_shape::evaluate(context, &mut findings);
    hygiene::evaluate(context, &mut findings);
    test_placement::evaluate(context, &mut findings);
    evidence::evaluate(context, &mut findings);
    size::evaluate(context, &mut findings);
    findings
}
