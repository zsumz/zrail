//! One invocation produces one structured result across every feasible candidate.

use zrail_core::{AnalysisQuality, MacroBindingMode, MacroExpansionAllow};

use crate::cargo::ResolvedCargoGraph;
use crate::source::{MacroCandidate, MacroExpansionFact, MacroOrigin, SourceIndex};

use super::{allowances::AllowanceIndex, bindings, failure::MacroBindingFailure, source};

pub(super) enum MacroBindingResult<'a> {
    Bound {
        allowances: Vec<&'a MacroExpansionAllow>,
        confidence: AnalysisQuality,
    },
    NoNameMatch,
    Rejected {
        attempted: Vec<&'a MacroExpansionAllow>,
        reasons: Vec<MacroBindingFailure>,
    },
}

pub(super) fn review<'a>(
    source: &SourceIndex,
    resolved_cargo: Option<&ResolvedCargoGraph>,
    expansion: &MacroExpansionFact,
    allowed: &AllowanceIndex<'a>,
) -> MacroBindingResult<'a> {
    review_with(
        expansion,
        allowed,
        |candidate, allowance| source::failures(candidate, allowance, resolved_cargo),
        |candidate, allowance| bindings::failure(source, candidate, allowance),
    )
}

pub(super) fn review_without_definitions<'a>(
    expansion: &MacroExpansionFact,
    allowed: &AllowanceIndex<'a>,
) -> MacroBindingResult<'a> {
    review_with(expansion, allowed, |_, _| Vec::new(), |_, _| None)
}

fn review_with<'a>(
    expansion: &MacroExpansionFact,
    allowed: &AllowanceIndex<'a>,
    source_failures: impl Fn(&MacroCandidate, &MacroExpansionAllow) -> Vec<MacroBindingFailure>,
    definition_failure: impl Fn(&MacroCandidate, &MacroExpansionAllow) -> Option<MacroBindingFailure>,
) -> MacroBindingResult<'a> {
    let mut matched = Vec::new();
    let mut attempted = Vec::new();
    let mut reasons = Vec::new();
    for candidate in &expansion.candidates {
        let candidate_attempts = candidate
            .allowance_names(&expansion.name)
            .into_iter()
            .flat_map(|name| allowed.get(name).into_iter().flatten().copied())
            .collect::<Vec<_>>();
        attempted.extend(candidate_attempts.iter().copied());
        let Some(names) = candidate_names(expansion, candidate, allowed) else {
            if unresolved(candidate) {
                reasons.extend(unresolved_failures(candidate));
            }
            let mut missing = candidate
                .policy_names()
                .filter(|name| !allowed.contains(name))
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
            if let Some(allowances) = conservative_allowances(&names, allowed) {
                matched.extend(allowances);
            } else {
                reasons.extend(unresolved_failures(candidate));
                reasons.extend(names.iter().map(|name| {
                    MacroBindingFailure::ConfidenceNotGranted {
                        allowance: (*name).into(),
                    }
                }));
            }
            continue;
        }
        for name in names {
            let mut accepted = Vec::new();
            let mut failures = Vec::new();
            for allowance in allowed.get(name).into_iter().flatten().copied() {
                let mut rejected = source_failures(candidate, allowance);
                rejected.extend(definition_failure(candidate, allowance));
                if rejected.is_empty() {
                    accepted.push(allowance);
                } else {
                    failures.extend(rejected);
                }
            }
            if accepted.is_empty() {
                reasons.extend(failures);
            } else {
                matched.extend(accepted);
            }
        }
    }
    dedup_allowances(&mut attempted);
    if attempted.is_empty() {
        return MacroBindingResult::NoNameMatch;
    }
    reasons.sort();
    reasons.dedup();
    if !reasons.is_empty() {
        return MacroBindingResult::Rejected { attempted, reasons };
    }
    dedup_allowances(&mut matched);
    MacroBindingResult::Bound {
        allowances: matched,
        confidence: expansion.quality,
    }
}

pub(super) fn candidate_names<'a>(
    expansion: &'a MacroExpansionFact,
    candidate: &'a MacroCandidate,
    allowed: &AllowanceIndex<'_>,
) -> Option<Vec<&'a str>> {
    let names = candidate.policy_names().collect::<Vec<_>>();
    let written_alias =
        names.len() == 1 && candidate.written_alias && allowed.contains(expansion.name.as_str());
    let conservative_fallback = unresolved(candidate)
        && allowed
            .get(expansion.name.as_str())
            .is_some_and(|allowances| {
                allowances
                    .iter()
                    .any(|allowance| allowance.binding == MacroBindingMode::Conservative)
            });
    if names.iter().all(|name| allowed.contains(name)) {
        Some(names)
    } else if written_alias || conservative_fallback {
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
            MacroOrigin::UnknownExportSet { reason } => {
                Some(MacroBindingFailure::UnknownExportSet {
                    candidate: candidate_name.clone(),
                    reason: reason.clone(),
                })
            }
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
                MacroOrigin::Pending { .. }
                    | MacroOrigin::UnknownExportSet { .. }
                    | MacroOrigin::Unresolved
            )
        })
}

fn conservative_allowances<'a>(
    names: &[&str],
    allowed: &AllowanceIndex<'a>,
) -> Option<Vec<&'a MacroExpansionAllow>> {
    let mut matched = Vec::new();
    for name in names {
        let conservative = allowed
            .get(name)?
            .iter()
            .filter(|allowance| allowance.binding == MacroBindingMode::Conservative)
            .copied()
            .collect::<Vec<_>>();
        let [allowance] = conservative.as_slice() else {
            return None;
        };
        matched.push(*allowance);
    }
    (!matched.is_empty()).then_some(matched)
}

fn dedup_allowances(values: &mut Vec<&MacroExpansionAllow>) {
    let mut retained = Vec::<*const MacroExpansionAllow>::new();
    values.retain(|allowance| {
        let pointer = std::ptr::from_ref(*allowance);
        if retained.contains(&pointer) {
            false
        } else {
            retained.push(pointer);
            true
        }
    });
}
