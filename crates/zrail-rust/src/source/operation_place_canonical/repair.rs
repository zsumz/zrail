//! Per-operation canonical place repair from domain-local declarations.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    CompilationDomain, ObservedFact, SourceIndex, SyntaxGuard,
    operation_model::{FieldPlaceFact, SourceOperationFact},
    operation_place_domains::{
        Candidate, Support, available_domains, candidates_at, has_projection, missing_domains,
        normalize, prefer_projected,
    },
};
use super::{
    catalog::Catalog,
    routes::{advance, declaring},
};

pub(crate) fn apply(
    index: &mut SourceIndex,
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
) {
    let catalog = Catalog::collect(&index.files, compilation_domains);
    for file in &mut index.files {
        let domains = compilation_domains.get(&file.relative);
        let paths = &file.paths;
        for operation in &mut file.operations {
            repair(operation, paths, domains, &catalog);
        }
    }
}

fn repair(
    operation: &mut SourceOperationFact,
    paths: &[ObservedFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
    catalog: &Catalog,
) {
    let Some(place) = operation.place.clone() else {
        return;
    };
    let Some((last, intermediates)) = place.fields.split_last() else {
        return;
    };
    let expected = available_domains(domains, &[&operation.identity.guard])
        .into_keys()
        .collect::<BTreeSet<_>>();
    let mut candidates = base_candidates(&place, paths, domains, &operation.identity.guard);
    if let Some(base) = exact_base(&candidates, &expected)
        && let Some(place) = &mut operation.place
    {
        place.base_name = base;
        place.base_quality = AnalysisQuality::Exact;
        place.base_file_local = false;
    }
    if candidates.is_empty() || (place.base_file_local && !has_projection(&candidates)) {
        operation.identity.quality = AnalysisQuality::Unresolved;
        return;
    }
    let mut unresolved = missing_domains(&expected, &candidates);
    for field in intermediates {
        let (next, missing) = advance(&candidates, field, catalog);
        unresolved.extend(missing);
        candidates = next;
        if candidates.is_empty() {
            operation.identity.quality = AnalysisQuality::Unresolved;
            return;
        }
    }
    let (declaring, missing) = declaring(&candidates, last, catalog);
    unresolved.extend(missing);
    if declaring.is_empty() {
        operation.identity.quality = AnalysisQuality::Unresolved;
        return;
    }
    apply_candidates(operation, normalize(declaring), !unresolved.is_empty());
}

fn exact_base(candidates: &[Candidate], expected: &BTreeSet<CompilationDomain>) -> Option<String> {
    if !missing_domains(expected, candidates).is_empty()
        || candidates.iter().any(|candidate| {
            candidate
                .domains
                .values()
                .any(|support| support.quality != AnalysisQuality::Exact)
        })
    {
        return None;
    }
    let names = candidates
        .iter()
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut names = names.into_iter();
    let name = names.next()?;
    if names.next().is_some() {
        return None;
    }
    Some(name.into())
}

fn base_candidates(
    place: &FieldPlaceFact,
    paths: &[ObservedFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
    operation_guard: &SyntaxGuard,
) -> Vec<Candidate> {
    let mut projected = place
        .base_span
        .map(|span| candidates_at(paths, span, domains, operation_guard))
        .unwrap_or_default();
    prefer_projected(&mut projected);
    if has_projection(&projected) {
        return projected;
    }
    vec![Candidate {
        name: place.base_name.clone(),
        domains: available_domains(domains, &[operation_guard])
            .into_iter()
            .map(|(domain, quality)| {
                (
                    domain,
                    Support {
                        quality: quality.max(place.base_quality),
                        projected: false,
                    },
                )
            })
            .collect(),
    }]
}

fn apply_candidates(
    operation: &mut SourceOperationFact,
    candidates: Vec<Candidate>,
    unresolved: bool,
) {
    let quality = candidates
        .iter()
        .flat_map(|candidate| candidate.domains.values())
        .fold(AnalysisQuality::Exact, |quality, support| {
            quality.max(support.quality)
        })
        .max(if unresolved {
            AnalysisQuality::Unresolved
        } else {
            AnalysisQuality::Exact
        });
    match candidates.as_slice() {
        [] => {}
        [candidate] => {
            operation.identity.name.clone_from(&candidate.name);
            operation.identity.canonical.clear();
            operation.identity.quality = quality;
            operation.file_local = false;
        }
        _ => {
            operation.identity.canonical = candidates
                .into_iter()
                .map(|candidate| candidate.name)
                .collect();
            operation.identity.quality = quality.max(AnalysisQuality::Conservative);
            operation.file_local = false;
        }
    }
}
