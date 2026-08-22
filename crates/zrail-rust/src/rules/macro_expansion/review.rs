//! One invocation passes only when every feasible candidate is named and bound.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, MacroBindingMode, MacroExpansionAllow};

use crate::source::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin};

use super::source;

pub(super) enum Review<'a> {
    Allowed(Vec<&'a str>),
    Unbound,
    Unreviewed,
}

pub(super) fn review<'a>(
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> Review<'a> {
    let mut matched = Vec::new();
    let mut saw_named_candidate = false;
    for candidate in &expansion.candidates {
        let Some(names) = candidate_names(expansion, candidate, allowed) else {
            return Review::Unreviewed;
        };
        saw_named_candidate = true;
        if unresolved(candidate) {
            if !conservative_written_binding(expansion, candidate, &names, allowed) {
                return Review::Unbound;
            }
        } else if !names
            .iter()
            .all(|name| source::bound(candidate, allowed[name]))
        {
            return Review::Unbound;
        }
        matched.extend(names);
    }
    matched.sort_unstable();
    matched.dedup();
    if saw_named_candidate {
        Review::Allowed(matched)
    } else {
        Review::Unreviewed
    }
}

pub(super) fn candidate_names<'a>(
    expansion: &'a MacroExpansionFact,
    candidate: &'a MacroCandidate,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> Option<Vec<&'a str>> {
    let names = candidate.policy_names().collect::<Vec<_>>();
    if names.iter().all(|name| allowed.contains_key(name)) {
        Some(names)
    } else if names.len() == 1
        && candidate.written_alias
        && allowed.contains_key(expansion.name.as_str())
    {
        Some(vec![expansion.name.as_str()])
    } else {
        None
    }
}

fn unresolved(candidate: &MacroCandidate) -> bool {
    candidate.observation.quality == AnalysisQuality::Unresolved
        || candidate.origins.iter().any(|origin| {
            matches!(
                origin,
                MacroOrigin::Pending { .. } | MacroOrigin::Unresolved
            )
        })
}

fn conservative_written_binding(
    expansion: &MacroExpansionFact,
    candidate: &MacroCandidate,
    names: &[&str],
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> bool {
    candidate.derivation == MacroDerivation::Written
        && candidate.observation.name == expansion.name
        && names == [expansion.name.as_str()]
        && allowed[expansion.name.as_str()].binding == MacroBindingMode::Conservative
        && allowed[expansion.name.as_str()].source.is_none()
        && allowed[expansion.name.as_str()].definition.is_none()
}
