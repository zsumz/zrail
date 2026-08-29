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
    let Some((subject, item)) = occurrence(fact, source) else {
        return Vec::new();
    };
    let mut candidates = source
        .trait_bounds
        .iter()
        .filter(|bounds| equivalent(&bounds.subject, &subject))
        .flat_map(|bounds| {
            let projection = bounds.subject.projection().cloned().unwrap_or_default();
            bound_candidates(bounds, &item, &projection)
        })
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
            .flat_map(|bounds| bound_candidates(bounds, &item, &projection)),
    );
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(super) fn occurrence_projection(
    fact: &ObservedFact,
    source: &SourceInstance,
) -> Option<ProjectionIdentity> {
    occurrence(fact, source).and_then(|(subject, _)| subject.projection().cloned())
}

fn occurrence(fact: &ObservedFact, source: &SourceInstance) -> Option<(BoundSubject, String)> {
    let written = fact.written.as_deref().unwrap_or(&fact.name);
    let (receiver, item) = written.rsplit_once("::")?;
    let declared = source
        .generic_types
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let subject = BoundSubject::from_receiver(receiver, &declared)?;
    Some((subject, item.to_owned()))
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
            projection: projection.clone(),
            kind: AssociatedCandidateKind::TypeEquality,
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        });
    providers.chain(equalities).collect()
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}
