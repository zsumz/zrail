//! Generic scopes retain the traits that can provide associated identities.

use std::collections::{BTreeMap, BTreeSet};

use syn::Generics;

use super::super::{
    AssociatedCandidateKind, BoundSubject, GenericAssociatedCandidate, ProjectionIdentity,
    ProviderAuthority, TraitBoundFact, trait_bounds,
};
use super::FactVisitor;

#[derive(Debug)]
pub(in crate::source) struct GenericBoundScope {
    declared: BTreeSet<String>,
    bounds: Vec<TraitBoundFact>,
}

impl FactVisitor<'_> {
    pub(in crate::source) fn with_generics(
        &mut self,
        generics: &Generics,
        include_self: bool,
        visit: impl FnOnce(&mut Self),
    ) {
        self.with_generics_and_bounds(generics, include_self, Vec::new(), visit);
    }

    pub(in crate::source) fn with_generics_and_bounds(
        &mut self,
        generics: &Generics,
        include_self: bool,
        additional: Vec<TraitBoundFact>,
        visit: impl FnOnce(&mut Self),
    ) {
        let checkpoint = (self.generic_types.len(), self.generic_values.len());
        if include_self {
            self.generic_types.push("Self".into());
        }
        self.generic_types.extend(
            generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        self.generic_values.extend(
            generics
                .const_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        let mut scope = self.scope(generics, include_self);
        scope.bounds.extend(additional);
        trait_bounds::normalize(&mut scope.bounds);
        self.generic_bound_scopes.push(scope);
        visit(self);
        self.generic_bound_scopes.pop();
        self.generic_types.truncate(checkpoint.0);
        self.generic_values.truncate(checkpoint.1);
    }

    pub(in crate::source) fn with_fresh_generics(&mut self, visit: impl FnOnce(&mut Self)) {
        let inherited_types = std::mem::take(&mut self.generic_types);
        let inherited_values = std::mem::take(&mut self.generic_values);
        let inherited_bounds = std::mem::take(&mut self.generic_bound_scopes);
        visit(self);
        self.generic_types = inherited_types;
        self.generic_values = inherited_values;
        self.generic_bound_scopes = inherited_bounds;
    }

    pub(in crate::source) fn active_trait_bounds(&self) -> Vec<TraitBoundFact> {
        let mut effective = BTreeMap::<BoundSubject, TraitBoundFact>::new();
        for scope in &self.generic_bound_scopes {
            for declared in &scope.declared {
                effective.retain(|subject, _| visible(subject.root()) != visible(declared));
            }
            for fact in &scope.bounds {
                let key = fact.subject.clone();
                effective
                    .entry(key)
                    .and_modify(|existing| merge(existing, fact))
                    .or_insert_with(|| fact.clone());
            }
        }
        effective.into_values().collect()
    }

    pub(in crate::source) fn generic_associated_candidates(
        &self,
        written: &str,
    ) -> Vec<GenericAssociatedCandidate> {
        let Some((receiver, item)) = written.rsplit_once("::") else {
            return Vec::new();
        };
        let declared = self.generic_types.iter().cloned().collect::<BTreeSet<_>>();
        let Some(subject) = BoundSubject::from_receiver(receiver, &declared) else {
            return Vec::new();
        };
        self.generic_associated_candidates_for(&subject, item)
    }

    pub(in crate::source) fn generic_associated_candidates_for(
        &self,
        subject: &BoundSubject,
        item: &str,
    ) -> Vec<GenericAssociatedCandidate> {
        let active = self.active_trait_bounds();
        let mut resolved = active
            .iter()
            .filter(|fact| equivalent(&fact.subject, subject))
            .flat_map(|fact| candidates(fact, item, &ProjectionIdentity::default()))
            .collect::<Vec<_>>();
        let BoundSubject::Projection { root, projection } = subject else {
            return resolved;
        };
        resolved.extend(
            active
                .iter()
                .filter(|fact| {
                    equivalent(
                        &fact.subject,
                        &if visible(root) == "Self" {
                            BoundSubject::SelfType
                        } else {
                            BoundSubject::TypeParameter(root.clone())
                        },
                    )
                })
                .flat_map(|fact| candidates(fact, item, projection)),
        );
        resolved.sort();
        resolved.dedup();
        resolved
    }

    fn scope(&self, generics: &Generics, include_self: bool) -> GenericBoundScope {
        GenericBoundScope {
            declared: trait_bounds::declared(generics, include_self),
            bounds: trait_bounds::from_generics(
                generics,
                include_self,
                &self.syntax_guard(),
                &self.lexical_scope,
            ),
        }
    }
}

fn visible(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn equivalent(left: &BoundSubject, right: &BoundSubject) -> bool {
    match (left, right) {
        (BoundSubject::SelfType, BoundSubject::SelfType) => true,
        (BoundSubject::TypeParameter(left), BoundSubject::TypeParameter(right)) => {
            visible(left) == visible(right)
        }
        (
            BoundSubject::Projection {
                root: left_root,
                projection: left_projection,
            },
            BoundSubject::Projection {
                root: right_root,
                projection: right_projection,
            },
        ) => visible(left_root) == visible(right_root) && left_projection.matches(right_projection),
        _ => false,
    }
}

fn candidates(
    fact: &TraitBoundFact,
    item: &str,
    projection: &ProjectionIdentity,
) -> Vec<GenericAssociatedCandidate> {
    let providers = fact
        .providers
        .iter()
        .map(|provider| GenericAssociatedCandidate {
            name: format!("{}::{item}", provider.path),
            canonical: Vec::new(),
            quality: fact.quality.max(provider.quality()),
            projection: projection.clone(),
            kind: AssociatedCandidateKind::TraitProvider,
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        });
    let equalities = fact
        .equalities
        .iter()
        .map(|target| GenericAssociatedCandidate {
            name: format!("{}::{item}", target.path),
            canonical: Vec::new(),
            quality: fact.quality.max(target.quality()),
            projection: ProjectionIdentity::default(),
            kind: AssociatedCandidateKind::TypeEquality,
            provider_complete: false,
            provider_authorities: [ProviderAuthority::Unknown].into(),
        });
    providers.chain(equalities).collect()
}

fn merge(existing: &mut TraitBoundFact, fact: &TraitBoundFact) {
    existing.providers.extend(fact.providers.iter().cloned());
    existing.providers.sort();
    existing.providers.dedup();
    existing.equalities.extend(fact.equalities.iter().cloned());
    existing.equalities.sort();
    existing.equalities.dedup();
    existing.quality = existing.quality.max(fact.quality);
}
