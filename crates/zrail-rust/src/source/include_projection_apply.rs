//! Projection is planned in stable file order and committed only after full success.

mod findings;

use zrail_core::Finding;

use super::{
    ObservedFact, SourceIndex, SourceSyntax,
    include_binding_projection::{CallSite, FactKey, FactProjection, ProjectionRequest, project},
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit, ProjectionLimits},
    include_resolution_state::ResolutionUsage,
    parse::{MAX_FACTS_PER_FILE, fact_count},
};
use findings::{budget_exhausted, context_issue, unresolved};

struct FileProjection {
    index: usize,
    paths: FactProjection,
    calls: FactProjection,
}

impl IncludeBindings {
    #[cfg(test)]
    pub(super) fn apply(&self, index: &mut SourceIndex) -> Vec<Finding> {
        self.apply_with_contract_limits(index, &zrail_core::AnalysisLimits::default())
    }

    pub(super) fn apply_with_contract_limits(
        &self,
        index: &mut SourceIndex,
        limits: &zrail_core::AnalysisLimits,
    ) -> Vec<Finding> {
        let include_facts = index
            .files
            .iter()
            .filter(|file| self.instances.requires_projection(&file.relative))
            .map(fact_count)
            .sum();
        let metrics = self.instances.metrics();
        let include_limits = ProjectionLimits::for_contract(
            include_facts,
            metrics
                .base_contexts
                .saturating_add(metrics.derived_contexts),
            limits,
        );
        self.apply_with_limits(index, include_limits)
    }

    pub(super) fn apply_with_limits(
        &self,
        index: &mut SourceIndex,
        include_limits: ProjectionLimits,
    ) -> Vec<Finding> {
        let context_metrics = self.instances.metrics();
        index.analysis_metrics.base_contexts = context_metrics.base_contexts;
        index.analysis_metrics.derived_contexts = context_metrics.derived_contexts;
        if !self.instances.is_complete() {
            return self.instances.issues().iter().map(context_issue).collect();
        }
        let ordinary_facts = index
            .files
            .iter()
            .filter(|file| {
                !self.instances.requires_projection(&file.relative)
                    && self.requires_ordinary_resolution(file)
            })
            .map(fact_count)
            .sum();
        let mut ordinary_budget = ProjectionBudget::new(ProjectionLimits::for_input(
            ordinary_facts,
            context_metrics.base_contexts,
        ));
        let mut include_budget = ProjectionBudget::new(include_limits);
        let mut order = (0..index.files.len()).collect::<Vec<_>>();
        order.sort_by(|left, right| {
            index.files[*left]
                .relative
                .cmp(&index.files[*right].relative)
        });
        let mut planned = Vec::new();
        let mut findings = Vec::new();
        for file_index in order {
            let file = &index.files[file_index];
            let include_connected = self.instances.requires_projection(&file.relative);
            if !include_connected && !self.requires_ordinary_resolution(file) {
                continue;
            }
            if include_connected {
                index.analysis_metrics.projection_files =
                    index.analysis_metrics.projection_files.saturating_add(1);
            }
            let budget = if include_connected {
                &mut include_budget
            } else {
                &mut ordinary_budget
            };
            let Some(mut remaining_file_facts) = MAX_FACTS_PER_FILE.checked_sub(fact_count(file))
            else {
                return vec![budget_exhausted(ProjectionLimit::Facts)];
            };
            let mut uncertain = None;
            let project_expression = file.syntax == SourceSyntax::Expression;
            let call_sites = file
                .calls
                .iter()
                .map(|fact| {
                    (
                        fact.span,
                        fact.written.as_deref().unwrap_or(&fact.name).to_owned(),
                        fact.guard,
                    )
                })
                .collect::<std::collections::BTreeSet<CallSite>>();
            let paths = match project(
                &ProjectionRequest {
                    bindings: self,
                    file: &file.relative,
                    facts: &file.paths,
                    usage: ResolutionUsage::Path,
                    call_sites: &call_sites,
                    project_expression,
                },
                &mut uncertain,
                &mut *budget,
                &mut remaining_file_facts,
            ) {
                Ok(paths) => paths,
                Err(limit) => return vec![budget_exhausted(limit)],
            };
            let calls = match project(
                &ProjectionRequest {
                    bindings: self,
                    file: &file.relative,
                    facts: &file.calls,
                    usage: ResolutionUsage::Call,
                    call_sites: &call_sites,
                    project_expression,
                },
                &mut uncertain,
                &mut *budget,
                &mut remaining_file_facts,
            ) {
                Ok(calls) => calls,
                Err(limit) => return vec![budget_exhausted(limit)],
            };
            if let Some(span) = uncertain {
                findings.push(unresolved(Some(&file.relative), Some(span)));
            }
            planned.push(FileProjection {
                index: file_index,
                paths,
                calls,
            });
        }
        index.analysis_metrics.projection_work = include_budget.used_work();
        index.analysis_metrics.projected_facts = include_budget.retained_facts();
        for projection in planned {
            let file = &mut index.files[projection.index];
            apply_projection(&mut file.paths, projection.paths);
            apply_projection(&mut file.calls, projection.calls);
        }
        findings
    }
}

fn apply_projection(facts: &mut Vec<ObservedFact>, projection: FactProjection) {
    facts.retain(|fact| {
        !projection
            .removals
            .contains(&(fact.name.clone(), fact.span, fact.guard))
    });
    for fact in facts.iter_mut() {
        let key: FactKey = (fact.name.clone(), fact.span, fact.guard);
        if let Some(quality) = projection.qualities.get(&key) {
            fact.quality = *quality;
        }
    }
    facts.extend(projection.additions);
}

#[cfg(test)]
#[path = "include_projection_apply_test.rs"]
mod include_projection_apply_test;
