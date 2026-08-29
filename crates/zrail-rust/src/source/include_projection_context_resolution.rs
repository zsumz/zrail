//! Projection qualifiers resolve independently from candidate provider paths.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::super::{
    BoundSubject, GenericPathIdentity, GuardAvailability, ProjectionIdentity, SourceInstanceId,
    SyntaxGuard,
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    source_instance::SourceInstance,
};

pub(super) fn resolve(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    source: &SourceInstance,
    projection: &ProjectionIdentity,
    scope: &[zrail_core::SourceSpan],
    guard: &SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<(ProjectionIdentity, AnalysisQuality)>, ProjectionLimit> {
    let Some(qualifier) = &projection.qualifying_trait else {
        return Ok(vec![(projection.clone(), projection.quality())]);
    };
    if qualifier.is_current_trait_context() {
        return resolve_current_trait(bindings, instance, source, projection, scope, guard, budget);
    }
    let resolved = resolve_path(bindings, instance, &qualifier.path, scope, guard, budget)?;
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

fn resolve_current_trait(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    source: &SourceInstance,
    projection: &ProjectionIdentity,
    scope: &[zrail_core::SourceSpan],
    guard: &SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<(ProjectionIdentity, AnalysisQuality)>, ProjectionLimit> {
    let marker = GenericPathIdentity::current_trait_context();
    let context = source.guard.combine(guard);
    let mut resolved = BTreeMap::<ProjectionIdentity, AnalysisQuality>::new();
    for bound in source.trait_bounds.iter().filter(|bound| {
        matches!(
            &bound.subject,
            BoundSubject::TypeParameter(root) if root == &marker.path
        )
    }) {
        let availability = bound
            .guard
            .availability_for_domain(&context, &source.domain);
        if !availability.is_available() {
            continue;
        }
        let availability_quality = if availability == GuardAvailability::Possible {
            AnalysisQuality::Unresolved
        } else {
            AnalysisQuality::Exact
        };
        for provider in &bound.providers {
            let candidates =
                resolve_path(bindings, instance, &provider.path, scope, guard, budget)?;
            if candidates.is_empty() {
                let mut contextual = projection.clone();
                contextual.qualifying_trait = Some(provider.clone());
                insert(&mut resolved, contextual, AnalysisQuality::Unresolved);
                continue;
            }
            for candidate in candidates {
                let mut contextual = projection.clone();
                contextual.qualifying_trait = Some(provider.with_path(candidate.name));
                insert(
                    &mut resolved,
                    contextual,
                    bound
                        .quality
                        .max(provider.quality())
                        .max(availability_quality)
                        .max(candidate.quality),
                );
            }
        }
    }
    if resolved.is_empty() {
        return Ok(vec![(projection.clone(), AnalysisQuality::Unresolved)]);
    }
    Ok(resolved.into_iter().collect())
}

fn resolve_path(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    written: &str,
    scope: &[zrail_core::SourceSpan],
    guard: &SyntaxGuard,
    budget: &mut ProjectionBudget,
) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
    bindings.resolve_written(
        &WrittenResolveRequest {
            instance,
            written,
            scope,
            depth: 0,
            usage: ResolutionUsage::Type,
            guard,
            allow_implicit_prelude: true,
        },
        &mut ResolutionTrail::new(),
        budget,
    )
}

fn insert(
    resolved: &mut BTreeMap<ProjectionIdentity, AnalysisQuality>,
    projection: ProjectionIdentity,
    quality: AnalysisQuality,
) {
    resolved
        .entry(projection)
        .and_modify(|existing| *existing = (*existing).max(quality))
        .or_insert(quality);
}
