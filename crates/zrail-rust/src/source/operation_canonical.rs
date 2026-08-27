//! Operation subjects reuse guarded Rust binding truth after source projection.

mod identity;
mod resolution;
mod updates;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::Finding;

use super::{
    CompilationDomain, SourceIndex,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimits},
    operation_place_canonical::catalog::Catalog,
    parse::{MAX_FACTS_PER_FILE, fact_count},
};

pub(super) fn apply(
    index: &mut SourceIndex,
    bindings: &IncludeBindings,
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    limits: &zrail_core::AnalysisLimits,
) -> Vec<Finding> {
    let affected = index
        .files
        .iter()
        .map(|file| file.operations.len())
        .sum::<usize>();
    let metrics = bindings.instances.metrics();
    let mut budget = ProjectionBudget::new(ProjectionLimits::for_contract(
        affected,
        metrics
            .base_contexts
            .saturating_add(metrics.derived_contexts),
        limits,
    ));
    let catalog = Catalog::collect(&index.files, compilation_domains);
    let mut planned = Vec::with_capacity(index.files.len());
    let mut unresolved = BTreeSet::new();
    for file in &index.files {
        let mut operations = file.operations.clone();
        if let Err(limit) = identity::canonicalize(
            &mut operations,
            bindings,
            &file.relative,
            &mut budget,
            &mut unresolved,
        ) {
            return vec![super::include_projection_apply::budget_exhausted(limit)];
        }
        let Some(mut remaining) = MAX_FACTS_PER_FILE.checked_sub(fact_count(file)) else {
            return vec![super::include_projection_apply::budget_exhausted(
                super::include_projection_budget::ProjectionLimit::Facts,
            )];
        };
        remaining = remaining.saturating_add(
            operations
                .iter()
                .filter(|operation| operation.struct_update.is_some())
                .count(),
        );
        if let Err(limit) = updates::expand(
            &mut operations,
            bindings,
            &catalog,
            &file.relative,
            &mut budget,
            &mut remaining,
            &mut unresolved,
        ) {
            return vec![super::include_projection_apply::budget_exhausted(limit)];
        }
        planned.push(operations);
    }
    for (file, operations) in index.files.iter_mut().zip(planned) {
        file.operations = operations;
    }
    index.analysis_metrics.projection_work = index
        .analysis_metrics
        .projection_work
        .saturating_add(budget.used_work());
    index.analysis_metrics.projected_facts = index
        .analysis_metrics
        .projected_facts
        .saturating_add(budget.retained_facts());
    unresolved
        .into_iter()
        .map(|(file, span)| super::include_projection_apply::unresolved(Some(&file), span))
        .collect()
}
