//! One collector normalizes every Rust spelling of a trait bound.

use std::collections::BTreeSet;

use syn::{GenericParam, Generics, TypeParamBound, WherePredicate, spanned::Spanned};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{BoundSubject, SyntaxGuard, TraitBoundFact, fact::source_span, fact::written_path};

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
        .filter_map(|parameter| {
            fact(
                BoundSubject::TypeParameter(parameter.ident.to_string()),
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
                fact(
                    BoundSubject::from_type(&predicate.bounded_ty, &declared)?,
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

pub(super) fn associated_types(
    declaration: &syn::ItemTrait,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let mut facts = declaration
        .items
        .iter()
        .filter_map(|item| {
            let syn::TraitItem::Type(item) = item else {
                return None;
            };
            let item_guard = guard.combine(super::attributes::cfg_guard(&item.attrs));
            let providers = item.bounds.iter().filter_map(provider).collect::<Vec<_>>();
            let mut fact = explicit(
                BoundSubject::Projection {
                    root: "Self".into(),
                    qualifying_trait: None,
                    associated: vec![item.ident.to_string()],
                },
                providers,
                &item_guard,
                scope,
                source_span(item.ident.span()),
            );
            if !super::ordinary_binding_facts::replacement_macros(&item.attrs, &item_guard, scope)
                .is_empty()
            {
                fact.quality = AnalysisQuality::Unresolved;
            }
            Some(fact)
        })
        .collect::<Vec<_>>();
    normalize(&mut facts);
    facts
}

pub(super) fn explicit(
    subject: BoundSubject,
    providers: Vec<String>,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> TraitBoundFact {
    TraitBoundFact {
        subject,
        providers,
        quality: AnalysisQuality::Exact,
        guard: guard.clone(),
        lexical_scope: scope.to_vec(),
        span,
    }
}

fn fact(
    subject: BoundSubject,
    bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> Option<TraitBoundFact> {
    let providers = bounds.iter().filter_map(provider).collect::<Vec<_>>();
    (!providers.is_empty()).then(|| explicit(subject, providers, guard, scope, span))
}

fn provider(bound: &TypeParamBound) -> Option<String> {
    match bound {
        TypeParamBound::Trait(bound) if matches!(bound.modifier, syn::TraitBoundModifier::None) => {
            Some(written_path(&bound.path))
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
    }
}
