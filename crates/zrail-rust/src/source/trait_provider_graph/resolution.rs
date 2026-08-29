//! Graph inputs resolve paths while retaining projection generic identity.

use zrail_core::AnalysisQuality;

use super::super::super::{
    BoundSubject, GenericPathIdentity, ProjectionIdentity, SourceInstanceId, TraitBoundFact,
    include_bindings::{IncludeBindings, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    model::TraitDeclarationFact,
};

pub(super) struct ResolvedBound {
    pub(super) projections: Vec<ProjectionIdentity>,
    pub(super) providers: Vec<ResolvedPath>,
    pub(super) equalities: Vec<ResolvedPath>,
    pub(super) complete: bool,
}

pub(super) fn resolve_declaration(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    declaration: &TraitDeclarationFact,
    budget: &mut ProjectionBudget,
) -> Result<Vec<ResolvedPath>, ProjectionLimit> {
    resolve(
        bindings,
        instance,
        &declaration.trait_path,
        &declaration.lexical_scope,
        &declaration.guard,
        budget,
    )
}

pub(super) fn resolve_bound(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    bound: &TraitBoundFact,
    budget: &mut ProjectionBudget,
) -> Result<ResolvedBound, ProjectionLimit> {
    let (projections, projection_complete) = resolve_projection(bindings, instance, bound, budget)?;
    let (providers, providers_complete) =
        resolve_all(bindings, instance, &bound.providers, bound, budget)?;
    let (equalities, equalities_complete) =
        resolve_all(bindings, instance, &bound.equalities, bound, budget)?;
    Ok(ResolvedBound {
        projections,
        providers,
        equalities,
        complete: projection_complete && providers_complete && equalities_complete,
    })
}

fn resolve_projection(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    bound: &TraitBoundFact,
    budget: &mut ProjectionBudget,
) -> Result<(Vec<ProjectionIdentity>, bool), ProjectionLimit> {
    let projection = match &bound.subject {
        BoundSubject::Projection { projection, .. } => projection,
        BoundSubject::SelfType | BoundSubject::TypeParameter(_) => {
            return Ok((vec![ProjectionIdentity::default()], true));
        }
    };
    let Some(qualifier) = &projection.qualifying_trait else {
        return Ok((
            vec![projection.clone()],
            projection.quality() != AnalysisQuality::Unresolved,
        ));
    };
    let resolved = resolve(
        bindings,
        instance,
        &qualifier.path,
        &bound.lexical_scope,
        &bound.guard,
        budget,
    )?;
    let complete = !resolved.is_empty()
        && resolved
            .iter()
            .all(|candidate| candidate.quality != AnalysisQuality::Unresolved);
    if resolved.is_empty() {
        return Ok((vec![projection.clone()], false));
    }
    Ok((
        resolved
            .into_iter()
            .map(|candidate| {
                let mut projection = projection.clone();
                projection.qualifying_trait = Some(qualifier.with_path(candidate.name));
                projection
            })
            .collect(),
        complete && qualifier.quality() != AnalysisQuality::Unresolved,
    ))
}

fn resolve_all(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    identities: &[GenericPathIdentity],
    bound: &TraitBoundFact,
    budget: &mut ProjectionBudget,
) -> Result<(Vec<ResolvedPath>, bool), ProjectionLimit> {
    let mut resolved = Vec::new();
    let mut complete = true;
    for identity in identities {
        let candidates = resolve(
            bindings,
            instance,
            &identity.path,
            &bound.lexical_scope,
            &bound.guard,
            budget,
        )?;
        complete &= identity.quality() != AnalysisQuality::Unresolved
            && !candidates.is_empty()
            && candidates
                .iter()
                .all(|candidate| candidate.quality != AnalysisQuality::Unresolved);
        resolved.extend(candidates);
    }
    Ok((resolved, complete))
}

fn resolve(
    bindings: &IncludeBindings,
    instance: SourceInstanceId,
    written: &str,
    scope: &[zrail_core::SourceSpan],
    guard: &super::super::super::SyntaxGuard,
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
