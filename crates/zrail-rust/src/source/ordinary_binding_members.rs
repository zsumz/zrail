//! Associated values and enum variants retain their guarded namespace identity.

use syn::Type;
use zrail_core::SourceSpan;

use super::super::{
    BindingAnchor, BindingKind, ConstructorForm, ImportBindingFact, SyntaxGuard,
    ordinary_binding_facts::{
        BindingDraft, item_guard, path_text, push, quality, replacement_macros, visibility,
    },
};

pub(super) fn impl_values(
    bindings: &mut Vec<ImportBindingFact>,
    item: &syn::ItemImpl,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) {
    let Type::Path(self_type) = item.self_ty.as_ref() else {
        return;
    };
    if self_type.qself.is_some() {
        return;
    }
    let full = path_text(&self_type.path);
    let leaf = self_type
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string());
    let impl_guard = item_guard(&item.attrs, enclosing_guard);
    for associated in &item.items {
        let (name, attributes, associated_visibility) = match associated {
            syn::ImplItem::Const(value) => {
                (value.ident.to_string(), value.attrs.as_slice(), &value.vis)
            }
            syn::ImplItem::Fn(value) => (
                value.sig.ident.to_string(),
                value.attrs.as_slice(),
                &value.vis,
            ),
            _ => continue,
        };
        let guard = item_guard(attributes, &impl_guard);
        let mut macros = replacement_macros(&item.attrs, &guard, scope);
        macros.extend(replacement_macros(attributes, &guard, scope));
        macros.sort();
        macros.dedup();
        let mut subjects = vec![full.clone()];
        if let Some(leaf) = &leaf
            && leaf != &full
        {
            subjects.push(leaf.clone());
        }
        for subject in subjects {
            let target = format!("{subject}::{name}");
            push(
                bindings,
                BindingDraft {
                    name: Some(target.clone()),
                    target,
                    kind: BindingKind::LocalValue,
                    anchor: BindingAnchor::Lexical,
                    visibility: visibility(associated_visibility),
                    quality: quality(&item.attrs).max(quality(attributes)),
                    replacement_macros: macros.clone(),
                    guard: guard.clone(),
                    scope,
                },
            );
        }
    }
}

pub(super) fn enum_variants(
    bindings: &mut Vec<ImportBindingFact>,
    item: &syn::ItemEnum,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) {
    let enum_guard = item_guard(&item.attrs, enclosing_guard);
    for variant in &item.variants {
        let guard = item_guard(&variant.attrs, &enum_guard);
        let mut macros = replacement_macros(&item.attrs, &guard, scope);
        macros.extend(replacement_macros(&variant.attrs, &guard, scope));
        macros.sort();
        macros.dedup();
        let name = format!("{}::{}", item.ident, variant.ident);
        push(
            bindings,
            BindingDraft {
                name: Some(name.clone()),
                target: name,
                kind: BindingKind::LocalConstructor(constructor_form(&variant.fields)),
                anchor: BindingAnchor::Lexical,
                visibility: visibility(&item.vis),
                quality: quality(&item.attrs).max(quality(&variant.attrs)),
                replacement_macros: macros,
                guard,
                scope,
            },
        );
    }
}

const fn constructor_form(fields: &syn::Fields) -> ConstructorForm {
    match fields {
        syn::Fields::Named(_) => ConstructorForm::Named,
        syn::Fields::Unnamed(_) => ConstructorForm::Tuple,
        syn::Fields::Unit => ConstructorForm::Unit,
    }
}
