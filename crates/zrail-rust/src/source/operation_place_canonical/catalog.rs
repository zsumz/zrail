//! Canonical declarations indexed by exact compilation-domain support.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::super::{
    CfgPredicate, CompilationDomain, RustFileFacts, SyntaxGuard,
    operation_place_domains::{Candidate, Support, canonical_candidates_at},
};

pub(super) struct Declaration {
    pub(super) domains: BTreeMap<CompilationDomain, Support>,
    pub(super) members: BTreeMap<String, Member>,
    pub(super) fields: BTreeMap<String, Vec<Candidate>>,
}

#[derive(Clone)]
pub(super) struct Member {
    pub(super) domains: BTreeMap<CompilationDomain, Support>,
    pub(super) guard: SyntaxGuard,
}

#[derive(Clone)]
pub(in crate::source) struct NamedField {
    pub(in crate::source) name: String,
    pub(in crate::source) guard: SyntaxGuard,
    pub(in crate::source) quality: AnalysisQuality,
}

#[derive(Default)]
pub(in crate::source) struct Catalog(pub(super) BTreeMap<String, Vec<Declaration>>);

impl Catalog {
    pub(in crate::source) fn collect(
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
                let members = members(source_fields, domains, &declaration.guard);
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

    pub(in crate::source) fn named_fields(
        &self,
        identity: &str,
        domain: &CompilationDomain,
    ) -> Option<Vec<NamedField>> {
        let declarations = self
            .0
            .get(identity)?
            .iter()
            .filter(|declaration| declaration.domains.contains_key(domain))
            .collect::<Vec<_>>();
        if declarations.is_empty() {
            return None;
        }
        let ambiguity = if declarations.len() > 1 {
            AnalysisQuality::Conservative
        } else {
            AnalysisQuality::Exact
        };
        let mut fields = BTreeMap::<String, NamedField>::new();
        for declaration in declarations {
            let declared = declaration.domains.get(domain)?;
            for (name, member) in &declaration.members {
                let Some(support) = member.domains.get(domain) else {
                    continue;
                };
                let quality = declared.quality.max(support.quality).max(ambiguity);
                fields
                    .entry(name.clone())
                    .and_modify(|field| {
                        field.guard = union_guards(&field.guard, &member.guard);
                        field.quality = field
                            .quality
                            .max(quality)
                            .max(AnalysisQuality::Conservative);
                    })
                    .or_insert_with(|| NamedField {
                        name: name.clone(),
                        guard: member.guard.clone(),
                        quality,
                    });
            }
        }
        Some(fields.into_values().collect())
    }
}

fn members(
    fields: &[super::super::type_policy_model::TypeFieldFact],
    domains: Option<&BTreeSet<CompilationDomain>>,
    declaration_guard: &SyntaxGuard,
) -> BTreeMap<String, Member> {
    let mut members = BTreeMap::<String, Member>::new();
    for field in fields {
        let support = super::super::operation_place_domains::available_domains(
            domains,
            &[declaration_guard, &field.guard],
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
        .collect::<BTreeMap<_, _>>();
        members
            .entry(field.name.clone())
            .and_modify(|member| {
                member.guard = union_guards(&member.guard, &field.guard);
                for (domain, next) in &support {
                    member
                        .domains
                        .entry(domain.clone())
                        .and_modify(|current| current.quality = current.quality.max(next.quality))
                        .or_insert(*next);
                }
            })
            .or_insert_with(|| Member {
                domains: support,
                guard: field.guard.clone(),
            });
    }
    members
}

fn union_guards(left: &SyntaxGuard, right: &SyntaxGuard) -> SyntaxGuard {
    SyntaxGuard::from_predicate(CfgPredicate::any(vec![left.predicate(), right.predicate()]))
}
