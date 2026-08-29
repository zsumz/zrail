//! One collector normalizes every Rust spelling of a trait bound.

#[path = "trait_bounds/constraints.rs"]
mod constraints;

use std::collections::BTreeSet;

use syn::{GenericParam, Generics, TypeParamBound, WherePredicate, spanned::Spanned};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    AssociatedSegment, BoundSubject, GenericPathIdentity, ProjectionIdentity, SyntaxGuard,
    TraitBoundFact, fact::source_span,
};

pub(super) fn declared(generics: &Generics, include_self: bool) -> BTreeSet<String> {
    let mut declared = generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Type(parameter) => Some(parameter.ident.to_string()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if include_self {
        declared.insert("Self".into());
    }
    declared
}

pub(super) fn from_generics(
    generics: &Generics,
    include_self: bool,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let declared = declared(generics, include_self);
    let mut facts = generics
        .type_params()
        .flat_map(|parameter| {
            from_bounds(
                &BoundSubject::TypeParameter(parameter.ident.to_string()),
                &parameter.bounds,
                guard,
                scope,
                source_span(parameter.ident.span()),
            )
        })
        .collect::<Vec<_>>();
    facts.extend(
        generics
            .where_clause
            .iter()
            .flat_map(|clause| &clause.predicates)
            .filter_map(|predicate| {
                let WherePredicate::Type(predicate) = predicate else {
                    return None;
                };
                Some((
                    BoundSubject::from_type(&predicate.bounded_ty, &declared)?,
                    predicate,
                ))
            })
            .flat_map(|(subject, predicate)| {
                from_bounds(
                    &subject,
                    &predicate.bounds,
                    guard,
                    scope,
                    source_span(predicate.bounded_ty.span()),
                )
            }),
    );
    normalize(&mut facts);
    facts
}

pub(super) fn from_bounds(
    subject: &BoundSubject,
    bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> Vec<TraitBoundFact> {
    constraints::from_bounds(subject, bounds, guard, scope, span)
}

pub(super) fn associated_types(
    declaration: &syn::ItemTrait,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let qualifier = GenericPathIdentity::wildcard(declaration.ident.to_string());
    let mut facts = declaration
        .items
        .iter()
        .filter_map(|item| {
            let syn::TraitItem::Type(item) = item else {
                return None;
            };
            let item_guard = guard.combine(super::attributes::cfg_guard(&item.attrs));
            let providers = item.bounds.iter().filter_map(provider).collect::<Vec<_>>();
            let equalities = item
                .default
                .as_ref()
                .and_then(|(_, ty)| GenericPathIdentity::type_path(ty))
                .into_iter()
                .collect();
            let mut fact = TraitBoundFact {
                subject: BoundSubject::Projection {
                    root: "Self".into(),
                    projection: ProjectionIdentity {
                        qualifying_trait: Some(qualifier.clone()),
                        associated: vec![AssociatedSegment::declaration(
                            &item.ident,
                            !item.generics.params.is_empty(),
                        )],
                    },
                },
                providers,
                equalities,
                quality: AnalysisQuality::Exact,
                guard: item_guard.clone(),
                lexical_scope: scope.to_vec(),
                span: source_span(item.ident.span()),
            };
            if !super::ordinary_binding_facts::replacement_macros(&item.attrs, &item_guard, scope)
                .is_empty()
                || (item.default.is_some() && fact.equalities.is_empty())
            {
                fact.quality = AnalysisQuality::Unresolved;
            }
            Some(fact)
        })
        .collect::<Vec<_>>();
    normalize(&mut facts);
    facts
}

pub(super) fn impl_associated_types(
    implementation: &syn::ItemImpl,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let Some((negative, trait_path, _)) = &implementation.trait_ else {
        return Vec::new();
    };
    if negative.is_some() {
        return Vec::new();
    }
    let qualifier = GenericPathIdentity::trait_path(trait_path);
    let mut facts = implementation
        .items
        .iter()
        .filter_map(|item| {
            let syn::ImplItem::Type(item) = item else {
                return None;
            };
            let item_guard = guard.combine(super::attributes::cfg_guard(&item.attrs));
            let target = GenericPathIdentity::type_path(&item.ty);
            let mut quality = qualifier.quality();
            if target.is_none()
                || item.defaultness.is_some()
                || !super::ordinary_binding_facts::replacement_macros(
                    &item.attrs,
                    &item_guard,
                    scope,
                )
                .is_empty()
            {
                quality = AnalysisQuality::Unresolved;
            }
            Some(TraitBoundFact {
                subject: BoundSubject::Projection {
                    root: "Self".into(),
                    projection: ProjectionIdentity {
                        qualifying_trait: Some(qualifier.clone()),
                        associated: vec![AssociatedSegment::declaration(
                            &item.ident,
                            !item.generics.params.is_empty(),
                        )],
                    },
                },
                providers: Vec::new(),
                equalities: target.into_iter().collect(),
                quality,
                guard: item_guard,
                lexical_scope: scope.to_vec(),
                span: source_span(item.ident.span()),
            })
        })
        .collect::<Vec<_>>();
    normalize(&mut facts);
    facts
}

pub(super) fn explicit(
    subject: BoundSubject,
    providers: Vec<GenericPathIdentity>,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> TraitBoundFact {
    TraitBoundFact {
        subject,
        providers,
        equalities: Vec::new(),
        quality: AnalysisQuality::Exact,
        guard: guard.clone(),
        lexical_scope: scope.to_vec(),
        span,
    }
}

fn provider(bound: &TypeParamBound) -> Option<GenericPathIdentity> {
    match bound {
        TypeParamBound::Trait(bound) if matches!(bound.modifier, syn::TraitBoundModifier::None) => {
            Some(GenericPathIdentity::trait_path(&bound.path))
        }
        _ => None,
    }
}

pub(super) fn normalize(facts: &mut Vec<TraitBoundFact>) {
    facts.sort();
    facts.dedup();
    for fact in facts {
        fact.providers.sort();
        fact.providers.dedup();
        fact.equalities.sort();
        fact.equalities.dedup();
    }
}
