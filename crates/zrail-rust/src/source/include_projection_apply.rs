//! Projection is planned in stable file order and committed only after full success.

use zrail_core::{AnalysisQuality, Finding};

use super::{
    ObservedFact, SourceIndex, SourceInstanceIssue, SourceSyntax,
    include_binding_projection::{CallSite, FactKey, FactProjection, ProjectionRequest, project},
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit, ProjectionLimits},
    include_resolution_state::ResolutionUsage,
    parse::{MAX_FACTS_PER_FILE, fact_count},
};

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
        let affected_facts = index
            .files
            .iter()
            .filter(|file| self.instances.requires_projection(&file.relative))
            .map(fact_count)
            .sum();
        self.apply_with_limits(
            index,
            ProjectionLimits::for_contract(
                affected_facts,
                self.instances.metrics().derived_contexts,
                limits,
            ),
        )
    }

    pub(super) fn apply_with_limits(
        &self,
        index: &mut SourceIndex,
        limits: ProjectionLimits,
    ) -> Vec<Finding> {
        let context_metrics = self.instances.metrics();
        index.analysis_metrics.base_contexts = context_metrics.base_contexts;
        index.analysis_metrics.derived_contexts = context_metrics.derived_contexts;
        if !self.instances.is_complete() {
            return self.instances.issues().iter().map(context_issue).collect();
        }
        let mut budget = ProjectionBudget::new(limits);
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
            if !self.instances.requires_projection(&file.relative) {
                continue;
            }
            index.analysis_metrics.projection_files =
                index.analysis_metrics.projection_files.saturating_add(1);
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
                &mut budget,
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
                &mut budget,
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
        index.analysis_metrics.projection_work = budget.used_work();
        index.analysis_metrics.projected_facts = budget.retained_facts();
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

fn unresolved(path: Option<&str>, span: Option<zrail_core::SourceSpan>) -> Finding {
    let mut finding = Finding::error(
        "RUST-INCLUDE-002",
        "rust.source.include-bindings",
        "source",
        "ordinary Rust path bindings could not be resolved completely",
    );
    if let Some(path) = path {
        finding = finding.at(path, span);
    }
    finding
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("reduce include or import indirection before trusting path and call authority")
}

fn context_issue(issue: &SourceInstanceIssue) -> Finding {
    let (id, message, path) = match issue {
        SourceInstanceIssue::DerivedContextLimit { used, limit, file } => (
            "RUST-CONTEXT-001",
            format!(
                "derived Rust source contexts reached {used}, exceeding the input-derived limit {limit}"
            ),
            Some(file.as_str()),
        ),
        SourceInstanceIssue::DepthLimit { file, depth, chain } => (
            "RUST-CONTEXT-002",
            format!(
                "Rust source context depth reached {depth} through {}",
                chain.join(" -> ")
            ),
            Some(file.as_str()),
        ),
        SourceInstanceIssue::Cycle { chain } => (
            "RUST-CONTEXT-003",
            format!("Rust source context cycle: {}", chain.join(" -> ")),
            chain.last().map(String::as_str),
        ),
    };
    let finding = Finding::error(id, "rust.source.contexts", "source", message)
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("remove the pathological source expansion before constructing lock authority");
    path.map_or(finding.clone(), |path| finding.at(path, None))
}

fn budget_exhausted(limit: ProjectionLimit) -> Finding {
    let (id, exhausted) = match limit {
        ProjectionLimit::Work => ("RUST-PROJECTION-001", "work"),
        ProjectionLimit::Facts => ("RUST-PROJECTION-002", "fact"),
    };
    Finding::error(
        id,
        "rust.source.include-bindings",
        "source",
        format!("repository-wide Rust binding projection exhausted its {exhausted} safety budget"),
    )
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help(
        "reduce namespace occurrences or binding indirection before trusting source authority",
    )
}

#[cfg(test)]
#[path = "include_projection_apply_test.rs"]
mod include_projection_apply_test;
