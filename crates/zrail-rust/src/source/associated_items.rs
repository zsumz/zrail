//! Associated items retain type and trait identity independently of placement.

use syn::{Item, Type, spanned::Spanned};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    SyntaxGuard,
    fact::{source_span, written_path},
    macro_binding_policy::MacroOccurrence,
    ordinary_binding_facts::{item_guard, quality, replacement_macros},
};

#[derive(Clone, Debug)]
pub(crate) struct AssociatedItemFact {
    pub(crate) kind: AssociatedItemKind,
    pub(crate) quality: AnalysisQuality,
    pub(crate) quality_without_macros: AnalysisQuality,
    pub(crate) replacement_macros: Vec<MacroOccurrence>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) span: SourceSpan,
}

#[derive(Clone, Debug)]
pub(crate) enum AssociatedItemKind {
    Implementation {
        self_type: String,
        trait_path: Option<String>,
        item: Option<String>,
    },
    TraitDefault {
        trait_path: String,
        item: String,
    },
}

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<AssociatedItemFact> {
    items
        .flat_map(|item| match item {
            Item::Impl(item) => impl_items(item, enclosing_guard, scope),
            Item::Trait(item) => trait_defaults(item, enclosing_guard, scope),
            _ => Vec::new(),
        })
        .collect()
}

fn impl_items(
    implementation: &syn::ItemImpl,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<AssociatedItemFact> {
    let Type::Path(self_type) = implementation.self_ty.as_ref() else {
        return Vec::new();
    };
    if self_type.qself.is_some() {
        return Vec::new();
    }
    let trait_path = match &implementation.trait_ {
        Some((negative, path, _)) if negative.is_none() => Some(written_path(path)),
        Some(_) => return Vec::new(),
        None => None,
    };
    let impl_guard = item_guard(&implementation.attrs, enclosing_guard);
    let self_name = written_path(&self_type.path);
    let mut facts = Vec::new();
    if trait_path.is_some() {
        facts.push(fact(
            AssociatedItemKind::Implementation {
                self_type: self_name.clone(),
                trait_path: trait_path.clone(),
                item: None,
            },
            &implementation.attrs,
            &[],
            &impl_guard,
            scope,
            source_span(self_type.path.span()),
        ));
    }
    facts.extend(implementation.items.iter().filter_map(|associated| {
        let (item, attributes, span) = match associated {
            syn::ImplItem::Const(value) => (
                value.ident.to_string(),
                value.attrs.as_slice(),
                value.ident.span(),
            ),
            syn::ImplItem::Fn(value) => (
                value.sig.ident.to_string(),
                value.attrs.as_slice(),
                value.sig.ident.span(),
            ),
            _ => return None,
        };
        Some(fact(
            AssociatedItemKind::Implementation {
                self_type: self_name.clone(),
                trait_path: trait_path.clone(),
                item: Some(item),
            },
            &implementation.attrs,
            attributes,
            &impl_guard,
            scope,
            source_span(span),
        ))
    }));
    facts
}

fn trait_defaults(
    declaration: &syn::ItemTrait,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<AssociatedItemFact> {
    let trait_guard = item_guard(&declaration.attrs, enclosing_guard);
    declaration
        .items
        .iter()
        .filter_map(|associated| {
            let (item, attributes, span) = match associated {
                syn::TraitItem::Const(value) if value.default.is_some() => (
                    value.ident.to_string(),
                    value.attrs.as_slice(),
                    value.ident.span(),
                ),
                syn::TraitItem::Fn(value) if value.default.is_some() => (
                    value.sig.ident.to_string(),
                    value.attrs.as_slice(),
                    value.sig.ident.span(),
                ),
                _ => return None,
            };
            Some(fact(
                AssociatedItemKind::TraitDefault {
                    trait_path: declaration.ident.to_string(),
                    item,
                },
                &declaration.attrs,
                attributes,
                &trait_guard,
                scope,
                source_span(span),
            ))
        })
        .collect()
}

fn fact(
    kind: AssociatedItemKind,
    outer_attributes: &[syn::Attribute],
    attributes: &[syn::Attribute],
    outer_guard: &SyntaxGuard,
    scope: &[SourceSpan],
    span: SourceSpan,
) -> AssociatedItemFact {
    let guard = item_guard(attributes, outer_guard);
    let mut macros = replacement_macros(outer_attributes, &guard, scope);
    macros.extend(replacement_macros(attributes, &guard, scope));
    macros.sort();
    macros.dedup();
    let base_quality = quality(outer_attributes).max(quality(attributes));
    AssociatedItemFact {
        kind,
        quality: if macros.is_empty() {
            base_quality
        } else {
            AnalysisQuality::Unresolved
        },
        quality_without_macros: base_quality,
        replacement_macros: macros,
        guard,
        lexical_scope: scope.to_vec(),
        span,
    }
}
