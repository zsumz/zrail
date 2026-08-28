//! Include instances resolve contextual `Self` and generic trait candidates.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    BoundSubject, GenericAssociatedCandidate, ObservedFact, ProviderAuthority, SourceInstanceId,
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
    let raw = if fact.associated_candidates.is_empty() && fact.inherits_parent_context {
        inherited_candidates(fact, source)
    } else {
        fact.associated_candidates.clone()
    };
    let mut resolved = BTreeMap::<(String, Vec<String>), GenericAssociatedCandidate>::new();
    for raw in raw {
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
            insert_candidate(&mut resolved, raw);
            continue;
        }
        for candidate in candidates {
            let authority = authority(&candidate);
            insert_candidate(
                &mut resolved,
                GenericAssociatedCandidate {
                    name: candidate.name,
                    canonical: Vec::new(),
                    quality: candidate.quality.max(raw.quality),
                    projection: raw.projection.clone(),
                    provider_complete: candidate.origin == ResolvedOrigin::CrateLocal,
                    provider_authorities: [authority].into(),
                },
            );
        }
    }
    Ok(resolved.into_values().collect())
}

fn inherited_candidates(
    fact: &ObservedFact,
    source: &SourceInstance,
) -> Vec<GenericAssociatedCandidate> {
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let Some((receiver, item)) = written.rsplit_once("::") else {
        return Vec::new();
    };
    let declared = source
        .generic_types
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let Some(subject) = BoundSubject::from_receiver(receiver, &declared) else {
        return Vec::new();
    };
    let mut candidates = source
        .trait_bounds
        .iter()
        .filter(|bounds| bounds.subject.without_qualifier() == subject.without_qualifier())
        .flat_map(|bounds| &bounds.providers)
        .map(|trait_path| GenericAssociatedCandidate {
            name: format!("{trait_path}::{item}"),
            canonical: Vec::new(),
            quality: AnalysisQuality::Exact,
            projection: Vec::new(),
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        })
        .collect::<Vec<_>>();
    let BoundSubject::Projection {
        root, associated, ..
    } = subject
    else {
        return candidates;
    };
    let root = if visible(&root) == "Self" {
        BoundSubject::SelfType
    } else {
        BoundSubject::TypeParameter(root)
    };
    candidates.extend(
        source
            .trait_bounds
            .iter()
            .filter(|bounds| bounds.subject.without_qualifier() == root)
            .flat_map(|bounds| &bounds.providers)
            .map(|trait_path| GenericAssociatedCandidate {
                name: format!("{trait_path}::{item}"),
                canonical: Vec::new(),
                quality: AnalysisQuality::Exact,
                projection: associated.clone(),
                provider_complete: false,
                provider_authorities: [ProviderAuthority::Unknown].into(),
            }),
    );
    candidates.sort();
    candidates.dedup();
    candidates
}

fn insert_candidate(
    candidates: &mut BTreeMap<(String, Vec<String>), GenericAssociatedCandidate>,
    candidate: GenericAssociatedCandidate,
) {
    let key = (candidate.name.clone(), candidate.projection.clone());
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
