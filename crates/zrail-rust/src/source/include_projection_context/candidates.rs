//! Included fragments inherit only matching generic associated constraints.

use std::collections::BTreeSet;

use crate::source::{
    AssociatedCandidateKind, BoundSubject, GenericAssociatedCandidate, ObservedFact,
    ProjectionIdentity, ProviderAuthority, TraitBoundFact, source_instance::SourceInstance,
};

pub(super) fn inherited(
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
        .filter(|bounds| equivalent(&bounds.subject, &subject))
        .flat_map(|bounds| bound_candidates(bounds, item, &ProjectionIdentity::default()))
        .collect::<Vec<_>>();
    let BoundSubject::Projection { root, projection } = subject else {
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
            .filter(|bounds| equivalent(&bounds.subject, &root))
            .flat_map(|bounds| bound_candidates(bounds, item, &projection)),
    );
    candidates.sort();
    candidates.dedup();
    candidates
}

fn equivalent(left: &BoundSubject, right: &BoundSubject) -> bool {
    match (left, right) {
        (BoundSubject::SelfType, BoundSubject::SelfType) => true,
        (BoundSubject::TypeParameter(left), BoundSubject::TypeParameter(right)) => {
            visible(left) == visible(right)
        }
        (
            BoundSubject::Projection {
                root: left_root,
                projection: left_projection,
            },
            BoundSubject::Projection {
                root: right_root,
                projection: right_projection,
            },
        ) => visible(left_root) == visible(right_root) && left_projection.matches(right_projection),
        _ => false,
    }
}

fn bound_candidates(
    bound: &TraitBoundFact,
    item: &str,
    projection: &ProjectionIdentity,
) -> Vec<GenericAssociatedCandidate> {
    let providers = bound
        .providers
        .iter()
        .map(|provider| GenericAssociatedCandidate {
            name: format!("{}::{item}", provider.path),
            canonical: Vec::new(),
            quality: bound.quality.max(provider.quality()),
            projection: projection.clone(),
            kind: AssociatedCandidateKind::TraitProvider,
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        });
    let equalities = bound
        .equalities
        .iter()
        .map(|target| GenericAssociatedCandidate {
            name: format!("{}::{item}", target.path),
            canonical: Vec::new(),
            quality: bound.quality.max(target.quality()),
            projection: ProjectionIdentity::default(),
            kind: AssociatedCandidateKind::TypeEquality,
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        });
    providers.chain(equalities).collect()
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}
