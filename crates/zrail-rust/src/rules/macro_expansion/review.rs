//! One invocation produces one structured result across every feasible candidate.

use std::collections::BTreeMap;

use zrail_core::{AnalysisQuality, MacroBindingMode, MacroExpansionAllow};

use crate::source::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin};

use super::{super::RuleContext, bindings, failure::MacroBindingFailure, source};

pub(super) enum MacroBindingResult<'a> {
    Bound {
        allowances: Vec<&'a str>,
        confidence: AnalysisQuality,
    },
    NoNameMatch,
    Rejected {
        attempted: Vec<&'a str>,
        reasons: Vec<MacroBindingFailure>,
    },
}

pub(super) fn review<'a>(
    context: &RuleContext<'_>,
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> MacroBindingResult<'a> {
    review_with(expansion, allowed, |candidate, allowance| {
        bindings::failure(context, candidate, allowance)
    })
}

pub(super) fn review_without_definitions<'a>(
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> MacroBindingResult<'a> {
    review_with(expansion, allowed, |_, _| None)
}

fn review_with<'a>(
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
    definition_failure: impl Fn(&MacroCandidate, &MacroExpansionAllow) -> Option<MacroBindingFailure>,
) -> MacroBindingResult<'a> {
    let mut matched = Vec::new();
    let mut attempted = Vec::new();
    let mut reasons = Vec::new();
    for candidate in &expansion.candidates {
        let candidate_attempts = candidate
            .allowance_names(&expansion.name)
            .into_iter()
            .filter(|name| allowed.contains_key(name))
            .collect::<Vec<_>>();
        attempted.extend(candidate_attempts.iter().copied());
        let Some(names) = candidate_names(expansion, candidate, allowed) else {
            let mut missing = candidate
                .policy_names()
                .filter(|name| !allowed.contains_key(name))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            missing.sort();
            missing.dedup();
            reasons.push(MacroBindingFailure::CandidateNotCovered {
                candidate: candidate_name(candidate),
                missing,
            });
            continue;
        };
        if unresolved(candidate) {
            if !conservative_written_binding(expansion, candidate, &names, allowed) {
                reasons.extend(unresolved_failures(candidate));
                reasons.extend(names.iter().map(|name| {
                    MacroBindingFailure::ConfidenceNotGranted {
                        allowance: (*name).into(),
                    }
                }));
            }
        } else {
            for name in &names {
                let allowance = allowed[name];
                reasons.extend(source::failures(candidate, allowance));
                reasons.extend(definition_failure(candidate, allowance));
            }
        }
        matched.extend(names);
    }
    attempted.sort_unstable();
    attempted.dedup();
    if attempted.is_empty() {
        return MacroBindingResult::NoNameMatch;
    }
    reasons.sort();
    reasons.dedup();
    if !reasons.is_empty() {
        return MacroBindingResult::Rejected { attempted, reasons };
    }
    matched.sort_unstable();
    matched.dedup();
    MacroBindingResult::Bound {
        allowances: matched,
        confidence: expansion.quality,
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

fn candidate_name(candidate: &MacroCandidate) -> String {
    let mut names = candidate.policy_names().collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        candidate.observation.name.clone()
    } else {
        names.join(" | ")
    }
}

fn unresolved_failures(candidate: &MacroCandidate) -> Vec<MacroBindingFailure> {
    let candidate_name = candidate_name(candidate);
    let mut failures = candidate
        .origins
        .iter()
        .filter_map(|origin| match origin {
            MacroOrigin::Pending { .. } => Some(MacroBindingFailure::PendingOrigin {
                candidate: candidate_name.clone(),
            }),
            MacroOrigin::Unresolved => Some(MacroBindingFailure::UnresolvedOrigin {
                candidate: candidate_name.clone(),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        failures.push(MacroBindingFailure::UnresolvedOrigin {
            candidate: candidate_name,
        });
    }
    failures
}

fn unresolved(candidate: &MacroCandidate) -> bool {
    candidate.origins.is_empty()
        || candidate.observation.quality == AnalysisQuality::Unresolved
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
