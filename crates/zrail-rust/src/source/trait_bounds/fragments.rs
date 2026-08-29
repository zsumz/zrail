//! Associated types declared inside include fragments remain lexical bounds.

use syn::{ImplItem, TraitItem, TypeParamBound};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::super::{
    AssociatedSegment, BoundSubject, GenericPathIdentity, ProjectionIdentity, SyntaxGuard,
    TraitBoundFact, attributes::cfg_guard, fact::source_span,
    ordinary_binding_facts::replacement_macros,
};

pub(in crate::source) fn fragment_impl_items(
    items: &[ImplItem],
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let mut facts = items
        .iter()
        .filter_map(|item| {
            let ImplItem::Type(item) = item else {
                return None;
            };
            let item_guard = guard.combine(cfg_guard(&item.attrs));
            let target = GenericPathIdentity::type_path(&item.ty);
            let mut quality = target
                .as_ref()
                .map_or(AnalysisQuality::Unresolved, GenericPathIdentity::quality);
            if item.defaultness.is_some()
                || !replacement_macros(&item.attrs, &item_guard, scope).is_empty()
            {
                quality = AnalysisQuality::Unresolved;
            }
            Some(TraitBoundFact {
                subject: projection(&item.ident, !item.generics.params.is_empty()),
                providers: Vec::new(),
                equalities: target.into_iter().collect(),
                quality,
                guard: item_guard,
                lexical_scope: scope.to_vec(),
                span: source_span(item.ident.span()),
            })
        })
        .collect::<Vec<_>>();
    super::normalize(&mut facts);
    facts
}

pub(in crate::source) fn fragment_trait_items(
    items: &[TraitItem],
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<TraitBoundFact> {
    let mut facts = items
        .iter()
        .filter_map(|item| {
            let TraitItem::Type(item) = item else {
                return None;
            };
            let item_guard = guard.combine(cfg_guard(&item.attrs));
            let providers = item.bounds.iter().filter_map(provider).collect::<Vec<_>>();
            let target = item
                .default
                .as_ref()
                .and_then(|(_, ty)| GenericPathIdentity::type_path(ty));
            let mut quality = providers
                .iter()
                .fold(AnalysisQuality::Exact, |quality, provider| {
                    quality.max(provider.quality())
                });
            if (item.default.is_some() && target.is_none())
                || !replacement_macros(&item.attrs, &item_guard, scope).is_empty()
            {
                quality = AnalysisQuality::Unresolved;
            }
            Some(TraitBoundFact {
                subject: projection(&item.ident, !item.generics.params.is_empty()),
                providers,
                equalities: target.into_iter().collect(),
                quality,
                guard: item_guard,
                lexical_scope: scope.to_vec(),
                span: source_span(item.ident.span()),
            })
        })
        .collect::<Vec<_>>();
    super::normalize(&mut facts);
    facts
}

fn projection(ident: &syn::Ident, has_generics: bool) -> BoundSubject {
    BoundSubject::Projection {
        root: "Self".into(),
        projection: ProjectionIdentity {
            qualifying_trait: Some(GenericPathIdentity::current_trait_context()),
            associated: vec![AssociatedSegment::declaration(ident, has_generics)],
        },
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
