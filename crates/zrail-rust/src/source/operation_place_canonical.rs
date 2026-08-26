//! Projected type declarations repair exact field places across source files.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::{
    CompilationDomain, ObservedFact, RustFileFacts, SourceIndex, SyntaxGuard,
    operation_model::{FieldPlaceFact, SourceOperationFact},
    operation_place_domains::{
        Candidate, Support, available_domains, candidates_at, canonical_candidates_at,
        has_projection, missing_domains, normalize, prefer_projected,
    },
};

struct Declaration {
    domains: BTreeMap<CompilationDomain, Support>,
    fields: BTreeMap<String, Vec<Candidate>>,
}

#[derive(Default)]
struct Catalog(BTreeMap<String, Vec<Declaration>>);

pub(super) fn apply(
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

impl Catalog {
    fn collect(
        files: &[RustFileFacts],
        compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    ) -> Self {
        let mut catalog = Self::default();
        for file in files {
            let domains = compilation_domains.get(&file.relative);
            for declaration in &file.type_policy.declarations {
                let Some(fields) = &declaration.fields else {
                    continue;
                };
                let identities = canonical_candidates_at(
                    &file.paths,
                    declaration.identity_span,
                    domains,
                    &declaration.guard,
                );
                let fields = fields
                    .iter()
                    .filter_map(|field| {
                        let span = field.type_shape.nominal_path_span()?;
                        let candidates =
                            canonical_candidates_at(&file.paths, span, domains, &declaration.guard);
                        (!candidates.is_empty()).then(|| (field.name.clone(), candidates))
                    })
                    .collect::<BTreeMap<_, _>>();
                for identity in identities {
                    catalog
                        .0
                        .entry(identity.name)
                        .or_default()
                        .push(Declaration {
                            domains: identity.domains,
                            fields: fields.clone(),
                        });
                }
            }
        }
        catalog
    }
}

fn repair(
    operation: &mut SourceOperationFact,
    paths: &[ObservedFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
    catalog: &Catalog,
) {
    let Some(place) = &operation.place else {
        return;
    };
    let Some((last, intermediates)) = place.fields.split_last() else {
        return;
    };
    let expected = available_domains(domains, &[&operation.identity.guard])
        .into_keys()
        .collect::<BTreeSet<_>>();
    let mut candidates = base_candidates(place, paths, domains, &operation.identity.guard);
    if candidates.is_empty() || (place.base_file_local && !has_projection(&candidates)) {
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
    for candidate in &mut candidates {
        candidate.name.push_str("::");
        candidate.name.push_str(last);
    }
    apply_candidates(operation, normalize(candidates), !unresolved.is_empty());
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

fn advance(
    candidates: &[Candidate],
    field: &str,
    catalog: &Catalog,
) -> (Vec<Candidate>, BTreeSet<CompilationDomain>) {
    let mut next = Vec::new();
    let mut unresolved = BTreeSet::new();
    for candidate in candidates {
        let Some(declarations) = catalog.0.get(&candidate.name) else {
            unresolved.extend(candidate.domains.keys().cloned());
            continue;
        };
        for (domain, base) in &candidate.domains {
            let routes = declarations
                .iter()
                .filter_map(|declaration| {
                    let declared = declaration.domains.get(domain)?;
                    let fields = declaration.fields.get(field)?;
                    Some((declared, fields))
                })
                .flat_map(|(declared, fields)| {
                    fields.iter().filter_map(move |field_type| {
                        field_type
                            .domains
                            .get(domain)
                            .map(|field_support| (field_type, declared, field_support))
                    })
                })
                .collect::<Vec<_>>();
            if routes.is_empty() {
                unresolved.insert(domain.clone());
                continue;
            }
            let ambiguity = if routes.len() > 1 {
                AnalysisQuality::Conservative
            } else {
                AnalysisQuality::Exact
            };
            for (field_type, declared, field_support) in routes {
                next.push(Candidate {
                    name: field_type.name.clone(),
                    domains: BTreeMap::from([(
                        domain.clone(),
                        Support {
                            quality: base
                                .quality
                                .max(declared.quality)
                                .max(field_support.quality)
                                .max(ambiguity),
                            projected: base.projected || field_support.projected,
                        },
                    )]),
                });
            }
        }
    }
    (normalize(next), unresolved)
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
