//! Occurrence provenance decides which associated declaration can terminate a candidate.

use zrail_core::AnalysisQuality;

use super::{Catalog, Key, TraitIdentity};
use crate::source::{
    CfgPredicate, GuardAvailability, SyntaxGuard,
    include_bindings::ResolvedTerminal,
    operation_canonical::{qualification::TraitSelection, resolution::Route},
};

pub(super) fn classify(
    catalog: &Catalog,
    route: &mut Route,
    context: &SyntaxGuard,
    selection: TraitSelection<'_>,
) {
    if route.terminal != ResolvedTerminal::Unknown
        && !matches!(selection, TraitSelection::Explicit(Some(_)))
    {
        return;
    }
    let Some((self_type, item)) = route.name.rsplit_once("::") else {
        return;
    };
    let key = Key {
        domain: route.domain.clone(),
        self_type: self_type.into(),
        item: item.into(),
    };
    let Some(traits) = catalog.entries.get(&key) else {
        return;
    };
    let guards = match selection {
        TraitSelection::Ordinary => {
            if catalog.external_self.contains(&key) {
                return;
            }
            traits.values().flatten().collect::<Vec<_>>()
        }
        TraitSelection::Explicit(None) => return,
        TraitSelection::Explicit(Some(name)) => traits
            .get(&TraitIdentity::Canonical(name.into()))
            .into_iter()
            .flatten()
            .collect(),
    };
    let union = SyntaxGuard::from_predicate(CfgPredicate::any(
        guards.into_iter().map(SyntaxGuard::predicate).collect(),
    ));
    if union.availability_for_domain(context, &route.domain) == GuardAvailability::Exact {
        route.terminal = ResolvedTerminal::Value;
        route.quality = AnalysisQuality::Exact;
    }
}
