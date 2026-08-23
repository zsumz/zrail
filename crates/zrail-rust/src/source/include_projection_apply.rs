//! Projection is planned in stable file order and committed only after full success.

use zrail_core::{AnalysisQuality, Finding};

use super::{
    ObservedFact, RustFileFacts, SourceIndex,
    include_bindings::{IncludeBindings, project},
    include_projection_budget::{ProjectionBudget, ProjectionLimit, ProjectionLimits},
    parse::{MAX_FACTS_PER_FILE, fact_count},
};

struct FileProjection {
    index: usize,
    paths: Vec<ObservedFact>,
    calls: Vec<ObservedFact>,
}

impl IncludeBindings {
    pub(super) fn apply(&self, index: &mut SourceIndex) -> Vec<Finding> {
        self.apply_with_limits(index, ProjectionLimits::default())
    }

    pub(super) fn apply_with_limits(
        &self,
        index: &mut SourceIndex,
        limits: ProjectionLimits,
    ) -> Vec<Finding> {
        if !self.instances.complete {
            return index
                .files
                .iter()
                .any(has_written_facts)
                .then(|| unresolved(None, None))
                .into_iter()
                .collect();
        }
        let mut budget = match ProjectionBudget::for_files(&index.files, limits) {
            Ok(budget) => budget,
            Err(limit) => return vec![budget_exhausted(limit)],
        };
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
            let Some(mut remaining_file_facts) = MAX_FACTS_PER_FILE.checked_sub(fact_count(file))
            else {
                return vec![budget_exhausted(ProjectionLimit::Facts)];
            };
            let mut uncertain = None;
            let paths = match project(
                self,
                &file.relative,
                &file.paths,
                &mut uncertain,
                &mut budget,
                &mut remaining_file_facts,
            ) {
                Ok(paths) => paths,
                Err(limit) => return vec![budget_exhausted(limit)],
            };
            let calls = match project(
                self,
                &file.relative,
                &file.calls,
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
        for projection in planned {
            index.files[projection.index].paths.extend(projection.paths);
            index.files[projection.index].calls.extend(projection.calls);
        }
        findings
    }
}

fn has_written_facts(file: &RustFileFacts) -> bool {
    file.paths
        .iter()
        .chain(&file.calls)
        .any(|fact| fact.written.is_some())
}

fn unresolved(path: Option<&str>, span: Option<zrail_core::SourceSpan>) -> Finding {
    let mut finding = Finding::error(
        "RUST-INCLUDE-002",
        "rust.source.include-bindings",
        "source",
        "include-spliced ordinary path bindings could not be resolved completely",
    );
    if let Some(path) = path {
        finding = finding.at(path, span);
    }
    finding
        .with_analysis(AnalysisQuality::Unresolved)
        .with_help("reduce include or import indirection before trusting path and call authority")
}

fn budget_exhausted(limit: ProjectionLimit) -> Finding {
    let exhausted = match limit {
        ProjectionLimit::Work => "work",
        ProjectionLimit::Facts => "fact",
    };
    Finding::error(
        "RUST-INCLUDE-002",
        "rust.source.include-bindings",
        "source",
        format!(
            "repository-wide include binding projection exhausted its {exhausted} safety budget"
        ),
    )
    .with_analysis(AnalysisQuality::Unresolved)
    .with_help("reduce include occurrences or binding indirection before trusting source authority")
}

#[cfg(test)]
#[path = "include_projection_apply_test.rs"]
mod include_projection_apply_test;
