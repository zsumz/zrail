//! Projection qualifiers resolve independently from candidate provider paths.

use zrail_core::AnalysisQuality;

use super::super::super::{
    ProjectionIdentity, SourceInstanceId, SyntaxGuard,
    include_bindings::IncludeBindings,
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
};

pub(super) fn resolve(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    projection: &ProjectionIdentity,
    scope: &[zrail_core::SourceSpan],
    guard: &SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<(ProjectionIdentity, AnalysisQuality)>, ProjectionLimit> {
    let Some(qualifier) = &projection.qualifying_trait else {
        return Ok(vec![(projection.clone(), projection.quality())]);
    };
    let resolved = bindings.resolve_written(
        &WrittenResolveRequest {
            instance,
            written: &qualifier.path,
            scope,
            depth: 0,
            usage: ResolutionUsage::Type,
            guard,
            allow_implicit_prelude: true,
        },
        &mut ResolutionTrail::new(),
        budget,
    )?;
    if resolved.is_empty() {
        return Ok(vec![(projection.clone(), AnalysisQuality::Unresolved)]);
    }
    Ok(resolved
        .into_iter()
        .map(|candidate| {
            let mut projection = projection.clone();
            projection.qualifying_trait = Some(qualifier.with_path(candidate.name));
            (projection, candidate.quality.max(qualifier.quality()))
        })
        .collect())
}
