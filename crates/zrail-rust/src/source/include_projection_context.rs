//! Include instances resolve contextual `Self` and generic trait candidates.

#[path = "include_projection_context/candidates.rs"]
mod candidates;
#[path = "include_projection_context_resolution.rs"]
mod projection_resolution;

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::{
    AssociatedCandidateKind, GenericAssociatedCandidate, ObservedFact, ProjectionIdentity,
    ProviderAuthority, SourceInstanceId,
    include_binding_helpers::unresolved,
    include_bindings::{IncludeBindings, ResolvedOrigin, ResolvedPath},
    include_projection_budget::{ProjectionBudget, ProjectionLimit},
    include_resolution_state::{ResolutionTrail, ResolutionUsage, WrittenResolveRequest},
    source_instance::SourceInstance,
};

pub(super) fn current_self(
    bindings: &IncludeBindings,
    fact: &ObservedFact,
    instance: SourceInstanceId,
    source: &SourceInstance,
    usage: ResolutionUsage,
    budget: &mut ProjectionBudget,
) -> Result<Option<Vec<ResolvedPath>>, ProjectionLimit> {
    if !fact.inherits_parent_context {
        return Ok(None);
    }
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let Some(suffix) = self_suffix(written) else {
        return Ok(None);
    };
    if suffix.matches("::").count() > 1 {
        return Ok(None);
    }
    let Some(identity) = &source.current_self else {
        return Ok(None);
    };
    let semantic = format!("{}{suffix}", identity.name);
    let mut resolved = bindings.resolve_written(
        &WrittenResolveRequest {
            instance,
            written: &semantic,
            scope: &fact.lexical_scope,
            depth: 0,
            usage,
            guard: &fact.guard,
            allow_implicit_prelude: false,
        },
        &mut ResolutionTrail::new(),
        budget,
    )?;
    if resolved.is_empty() {
        resolved.push(unresolved(&semantic));
    }
    for candidate in &mut resolved {
        candidate.quality = candidate.quality.max(identity.quality);
        if identity.file_local && candidate.origin != ResolvedOrigin::CrateLocal {
            candidate.quality = AnalysisQuality::Unresolved;
        }
        candidate.requires_projection = true;
    }
    Ok(Some(resolved))
}

pub(super) fn associated_candidates(
    bindings: &IncludeBindings,
    fact: &ObservedFact,
    instance: SourceInstanceId,
    source: &SourceInstance,
    budget: &mut ProjectionBudget,
) -> Result<Vec<GenericAssociatedCandidate>, ProjectionLimit> {
    let mut raw = fact.associated_candidates.clone();
    if fact.inherits_parent_context {
        raw.extend(candidates::inherited(fact, source));
    }
    raw.sort();
    raw.dedup();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let occurrence_projections =
        candidates::resolve_occurrence(bindings, fact, instance, source, budget)?;
    let mut resolved = BTreeMap::<
        (String, ProjectionIdentity, AssociatedCandidateKind),
        GenericAssociatedCandidate,
    >::new();
    for raw in raw {
        let projections = projection_resolution::resolve(
            bindings,
            instance,
            source,
            &raw.projection,
            &fact.lexical_scope,
            &fact.guard,
            budget,
        )?;
        let candidates = bindings.resolve_written(
            &WrittenResolveRequest {
                instance,
                written: &raw.name,
                scope: &fact.lexical_scope,
                depth: 0,
                usage: ResolutionUsage::Type,
                guard: &fact.guard,
                allow_implicit_prelude: true,
            },
            &mut ResolutionTrail::new(),
            budget,
        )?;
        if candidates.is_empty() {
            for (projection, quality) in projections {
                let Some(occurrence_quality) =
                    candidates::matching_quality(&projection, occurrence_projections.as_deref())
                else {
                    continue;
                };
                let mut candidate = raw.clone();
                candidate.projection =
                    candidates::retained_projection(raw.kind, &projection, &raw.name);
                candidate.quality = candidate.quality.max(quality).max(occurrence_quality);
                insert_candidate(&mut resolved, candidate);
            }
            continue;
        }
        for candidate in candidates {
            for (projection, projection_quality) in &projections {
                let Some(occurrence_quality) =
                    candidates::matching_quality(projection, occurrence_projections.as_deref())
                else {
                    continue;
                };
                let authority = authority(&candidate);
                let quality = candidate
                    .quality
                    .max(raw.quality)
                    .max(*projection_quality)
                    .max(occurrence_quality);
                insert_candidate(
                    &mut resolved,
                    GenericAssociatedCandidate {
                        name: candidate.name.clone(),
                        canonical: Vec::new(),
                        quality,
                        projection: candidates::retained_projection(
                            raw.kind,
                            projection,
                            &candidate.name,
                        ),
                        kind: raw.kind,
                        provider_complete: match raw.kind {
                            AssociatedCandidateKind::TraitProvider => {
                                candidate.origin == ResolvedOrigin::CrateLocal
                                    && quality != AnalysisQuality::Unresolved
                            }
                            AssociatedCandidateKind::TypeEquality => {
                                quality != AnalysisQuality::Unresolved
                            }
                        },
                        provider_authorities: [authority].into(),
                    },
                );
            }
        }
    }
    Ok(resolved.into_values().collect())
}

fn insert_candidate(
    candidates: &mut BTreeMap<
        (String, ProjectionIdentity, AssociatedCandidateKind),
        GenericAssociatedCandidate,
    >,
    candidate: GenericAssociatedCandidate,
) {
    let key = (
        candidate.name.clone(),
        candidate.projection.clone(),
        candidate.kind,
    );
    candidates
        .entry(key)
        .and_modify(|existing| {
            existing.quality = existing.quality.max(candidate.quality);
            existing.provider_complete &= candidate.provider_complete;
            existing
                .provider_authorities
                .extend(candidate.provider_authorities.iter().cloned());
        })
        .or_insert(candidate);
}

fn authority(candidate: &ResolvedPath) -> ProviderAuthority {
    match candidate.origin {
        ResolvedOrigin::CrateLocal => ProviderAuthority::LocalCrate,
        ResolvedOrigin::External => external_authority(&candidate.name),
        ResolvedOrigin::Unknown => ProviderAuthority::Unknown,
    }
}

fn external_authority(path: &str) -> ProviderAuthority {
    path.trim_start_matches("::")
        .split("::")
        .next()
        .filter(|root| !root.is_empty())
        .map_or(ProviderAuthority::Unknown, |root| {
            ProviderAuthority::ExternalRoot(visible(root).into())
        })
}

fn self_suffix(written: &str) -> Option<&str> {
    let suffix = written.strip_prefix("Self")?;
    (suffix.is_empty() || suffix.starts_with("::")).then_some(suffix)
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}
