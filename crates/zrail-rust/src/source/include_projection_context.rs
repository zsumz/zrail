//! Include instances resolve contextual `Self` and generic trait candidates.

use std::collections::BTreeMap;

use zrail_core::AnalysisQuality;

use super::super::{
    GenericAssociatedCandidate, ObservedFact, SourceInstanceId,
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
    let mut resolved = BTreeMap::<String, GenericAssociatedCandidate>::new();
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
            insert_candidate(
                &mut resolved,
                GenericAssociatedCandidate {
                    name: candidate.name,
                    canonical: Vec::new(),
                    quality: candidate.quality.max(raw.quality),
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
    source
        .generic_bounds
        .iter()
        .find(|bounds| visible_path(&bounds.parameter) == visible_path(receiver))
        .into_iter()
        .flat_map(|bounds| &bounds.traits)
        .map(|trait_path| GenericAssociatedCandidate {
            name: format!("{trait_path}::{item}"),
            canonical: Vec::new(),
            quality: AnalysisQuality::Exact,
        })
        .collect()
}

fn insert_candidate(
    candidates: &mut BTreeMap<String, GenericAssociatedCandidate>,
    candidate: GenericAssociatedCandidate,
) {
    candidates
        .entry(candidate.name.clone())
        .and_modify(|existing| existing.quality = existing.quality.max(candidate.quality))
        .or_insert(candidate);
}

fn self_suffix(written: &str) -> Option<&str> {
    let suffix = written.strip_prefix("Self")?;
    (suffix.is_empty() || suffix.starts_with("::")).then_some(suffix)
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn visible_path(path: &str) -> String {
    path.split("::").map(visible).collect::<Vec<_>>().join("::")
}
