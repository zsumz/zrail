//! Canonical declarations indexed by exact compilation-domain support.

use std::collections::{BTreeMap, BTreeSet};

use super::super::{
    CompilationDomain, RustFileFacts,
    operation_place_domains::{Candidate, Support, canonical_candidates_at},
};

pub(super) struct Declaration {
    pub(super) domains: BTreeMap<CompilationDomain, Support>,
    pub(super) members: BTreeMap<String, BTreeMap<CompilationDomain, Support>>,
    pub(super) fields: BTreeMap<String, Vec<Candidate>>,
}

#[derive(Default)]
pub(super) struct Catalog(pub(super) BTreeMap<String, Vec<Declaration>>);

impl Catalog {
    pub(super) fn collect(
        files: &[RustFileFacts],
        compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    ) -> Self {
        let mut catalog = Self::default();
        for file in files {
            let domains = compilation_domains.get(&file.relative);
            for declaration in &file.type_policy.declarations {
                let Some(source_fields) = &declaration.fields else {
                    continue;
                };
                let identities = canonical_candidates_at(
                    &file.paths,
                    declaration.identity_span,
                    domains,
                    &declaration.guard,
                );
                let fields = source_fields
                    .iter()
                    .filter_map(|field| {
                        let span = field.type_shape.nominal_path_span()?;
                        let candidates =
                            canonical_candidates_at(&file.paths, span, domains, &declaration.guard);
                        (!candidates.is_empty()).then(|| (field.name.clone(), candidates))
                    })
                    .collect::<BTreeMap<_, _>>();
                let members = source_fields
                    .iter()
                    .map(|field| {
                        let domains = super::super::operation_place_domains::available_domains(
                            domains,
                            &[&declaration.guard, &field.guard],
                        )
                        .into_iter()
                        .map(|(domain, quality)| {
                            (
                                domain,
                                Support {
                                    quality,
                                    projected: false,
                                },
                            )
                        })
                        .collect();
                        (field.name.clone(), domains)
                    })
                    .collect::<BTreeMap<_, _>>();
                for identity in identities {
                    catalog
                        .0
                        .entry(identity.name)
                        .or_default()
                        .push(Declaration {
                            domains: identity.domains,
                            members: members.clone(),
                            fields: fields.clone(),
                        });
                }
            }
        }
        catalog
    }
}
