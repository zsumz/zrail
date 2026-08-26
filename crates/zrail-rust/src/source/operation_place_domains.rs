//! Compilation-domain support keeps projected place candidates world-local.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, SourceSpan};

use super::{CompilationDomain, FactNamespace, GuardAvailability, ObservedFact, SyntaxGuard};

#[derive(Clone, Copy)]
pub(super) struct Support {
    pub(super) quality: AnalysisQuality,
    pub(super) projected: bool,
}

#[derive(Clone)]
pub(super) struct Candidate {
    pub(super) name: String,
    pub(super) domains: BTreeMap<CompilationDomain, Support>,
}

pub(super) fn candidates_at(
    paths: &[ObservedFact],
    span: SourceSpan,
    domains: Option<&BTreeSet<CompilationDomain>>,
    context_guard: &SyntaxGuard,
) -> Vec<Candidate> {
    let mut candidates = BTreeMap::<String, Candidate>::new();
    for fact in paths
        .iter()
        .filter(|fact| fact.span == Some(span) && fact.namespace == FactNamespace::Type)
    {
        let support = available_domains(domains, &[context_guard, &fact.guard]);
        for name in fact.policy_names() {
            let candidate = candidates.entry(name.into()).or_insert_with(|| Candidate {
                name: name.into(),
                domains: BTreeMap::new(),
            });
            for (domain, guard_quality) in &support {
                merge_support(
                    &mut candidate.domains,
                    domain.clone(),
                    Support {
                        quality: fact.quality.max(*guard_quality),
                        projected: fact.written.is_none() || !fact.canonical.is_empty(),
                    },
                );
            }
        }
    }
    candidates
        .into_values()
        .filter(|candidate| !candidate.domains.is_empty())
        .collect()
}

pub(super) fn canonical_candidates_at(
    paths: &[ObservedFact],
    span: SourceSpan,
    domains: Option<&BTreeSet<CompilationDomain>>,
    context_guard: &SyntaxGuard,
) -> Vec<Candidate> {
    let mut candidates = candidates_at(paths, span, domains, context_guard);
    prefer_projected(&mut candidates);
    candidates
}

pub(super) fn normalize(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut normalized = BTreeMap::<String, Candidate>::new();
    for candidate in candidates {
        let entry = normalized
            .entry(candidate.name.clone())
            .or_insert_with(|| candidate.clone());
        for (domain, support) in candidate.domains {
            merge_support(&mut entry.domains, domain, support);
        }
    }
    normalized.into_values().collect()
}

pub(super) fn has_projection(candidates: &[Candidate]) -> bool {
    candidates
        .iter()
        .any(|candidate| candidate.domains.values().any(|support| support.projected))
}

pub(super) fn available_domains(
    domains: Option<&BTreeSet<CompilationDomain>>,
    guards: &[&SyntaxGuard],
) -> BTreeMap<CompilationDomain, AnalysisQuality> {
    domains
        .into_iter()
        .flatten()
        .filter_map(|domain| {
            let guard = guards
                .iter()
                .fold(SyntaxGuard::Ordinary, |guard, next| guard.combine(*next));
            match guard.availability_in_domain(domain) {
                GuardAvailability::Absent => None,
                GuardAvailability::Exact => Some((domain.clone(), AnalysisQuality::Exact)),
                GuardAvailability::Possible => {
                    Some((domain.clone(), AnalysisQuality::Conservative))
                }
            }
        })
        .collect()
}

pub(super) fn prefer_projected(candidates: &mut Vec<Candidate>) {
    let projected = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .domains
                .iter()
                .filter(|(_, support)| support.projected)
                .map(|(domain, _)| domain.clone())
        })
        .collect::<BTreeSet<_>>();
    for candidate in candidates.iter_mut() {
        candidate
            .domains
            .retain(|domain, support| support.projected || !projected.contains(domain));
    }
    candidates.retain(|candidate| !candidate.domains.is_empty());
}

pub(super) fn missing_domains(
    expected: &BTreeSet<CompilationDomain>,
    candidates: &[Candidate],
) -> BTreeSet<CompilationDomain> {
    let observed = candidates
        .iter()
        .flat_map(|candidate| candidate.domains.keys().cloned())
        .collect::<BTreeSet<_>>();
    expected.difference(&observed).cloned().collect()
}

fn merge_support(
    domains: &mut BTreeMap<CompilationDomain, Support>,
    domain: CompilationDomain,
    support: Support,
) {
    match domains.get_mut(&domain) {
        Some(current) if current.projected && !support.projected => {}
        Some(current) if !current.projected && support.projected => *current = support,
        Some(current) => current.quality = current.quality.max(support.quality),
        None => {
            domains.insert(domain, support);
        }
    }
}

#[cfg(test)]
#[path = "operation_place_domains_test.rs"]
mod operation_place_domains_test;
