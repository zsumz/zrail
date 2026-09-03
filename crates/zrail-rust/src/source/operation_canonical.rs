//! Operation subjects reuse guarded Rust binding truth after source projection.

mod associated;
mod identity;
mod qualification;
mod resolution;
mod returns;
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
        .map(|file| file.operations.len() + file.associated_items.len())
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
    let associated = match associated::Catalog::collect(index, bindings, &mut budget) {
        Ok(catalog) => catalog,
        Err(limit) => return vec![super::include_projection_apply::budget_exhausted(limit)],
    };
    let mut resolver = resolution::Resolver::new(&mut budget);
    let mut planned = Vec::with_capacity(index.files.len());
    let mut unresolved = BTreeSet::new();
    for file in &index.files {
        let mut operations = file.operations.clone();
        let mut call_resolutions = Vec::new();
        if let Err(limit) = returns::canonicalize(
            &mut operations,
            bindings,
            &file.relative,
            file.syntax,
            &associated,
            &mut resolver,
        ) {
            return vec![super::include_projection_apply::budget_exhausted(limit)];
        }
        if let Err(limit) = identity::canonicalize(
            &mut operations,
            bindings,
            &file.relative,
            file.syntax,
            &associated,
            &mut resolver,
            &mut unresolved,
            &mut call_resolutions,
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
            file.syntax,
            &mut resolver,
            &mut remaining,
            &mut unresolved,
        ) {
            return vec![super::include_projection_apply::budget_exhausted(limit)];
        }
        planned.push((operations, call_resolutions));
    }
    drop(resolver);
    for (file, (operations, call_resolutions)) in index.files.iter_mut().zip(planned) {
        file.operations = operations;
        for boundary in call_resolutions {
            if !file.call_resolutions.contains(&boundary) {
                file.call_resolutions.push(boundary);
            }
        }
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
