//! Enum variants retain their guarded value-namespace identity.
use zrail_core::SourceSpan;

use super::super::{
    BindingAnchor, BindingKind, ConstructorForm, ImportBindingFact, SyntaxGuard,
    ordinary_binding_facts::{
        BindingDraft, item_guard, push, quality, replacement_macros, visibility,
    },
};

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
