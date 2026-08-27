//! Field traversal and declaring-type selection across exact domains.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    CompilationDomain,
    operation_place_domains::{Candidate, Support, normalize},
};
use super::catalog::Catalog;

pub(super) fn advance(
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

pub(super) fn declaring(
    candidates: &[Candidate],
    field: &str,
    catalog: &Catalog,
) -> (Vec<Candidate>, BTreeSet<CompilationDomain>) {
    let mut declaring = Vec::new();
    let mut unresolved = BTreeSet::new();
    for candidate in candidates {
        let declarations = catalog.0.get(&candidate.name);
        for (domain, base) in &candidate.domains {
            let routes = declarations
                .into_iter()
                .flatten()
                .filter_map(|declaration| {
                    let declared = declaration.domains.get(domain)?;
                    let member = declaration.members.get(field)?.domains.get(domain)?;
                    Some((declared, member))
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
            for (declared, member) in routes {
                declaring.push(Candidate {
                    name: format!("{}::{field}", candidate.name),
                    domains: BTreeMap::from([(
                        domain.clone(),
                        Support {
                            quality: base
                                .quality
                                .max(declared.quality)
                                .max(member.quality)
                                .max(ambiguity),
                            projected: base.projected,
                        },
                    )]),
                });
            }
        }
    }
    (normalize(declaring), unresolved)
}

#[cfg(test)]
#[path = "routes_test.rs"]
mod routes_test;
