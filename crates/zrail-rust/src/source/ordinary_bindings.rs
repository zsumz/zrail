//! Ordinary type-path bindings retain every guarded declaration in one lexical scope.

#[path = "ordinary_binding_members.rs"]
mod members;

use syn::{Item, Type};
use zrail_core::SourceSpan;

use super::{
    BindingAnchor, BindingKind, ConstructorForm, ImportBindingFact, SyntaxGuard,
    ordinary_binding_facts::{
        BindingContext, BindingDraft, foreign, item_guard, local, module, path_text, push, quality,
        replacement_macros, visibility,
    },
    ordinary_use_bindings::{UseBindingContext, collect_use},
};

use members::enum_variants;

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    enclosing_guard: &SyntaxGuard,
    lexical_scope: &[SourceSpan],
) -> Vec<ImportBindingFact> {
    let mut bindings = Vec::new();
    for item in items {
        match item {
            Item::Use(item) => {
                let guard = item_guard(&item.attrs, enclosing_guard);
                let context = UseBindingContext {
                    anchor: if item.leading_colon.is_some() {
                        BindingAnchor::Absolute
                    } else {
                        BindingAnchor::UsePath
                    },
                    visibility: visibility(&item.vis),
                    quality: quality(&item.attrs),
                    replacement_macros: replacement_macros(&item.attrs, &guard, lexical_scope),
                    guard,
                    scope: lexical_scope,
                };
                collect_use(&mut bindings, Vec::new(), &item.tree, &context);
            }
            Item::ExternCrate(item) => {
                let guard = item_guard(&item.attrs, enclosing_guard);
                let name = item
                    .rename
                    .as_ref()
                    .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                push(
                    &mut bindings,
                    BindingDraft {
                        name: Some(name),
                        target: item.ident.to_string(),
                        kind: BindingKind::Import,
                        anchor: if item.ident == "self" {
                            BindingAnchor::CrateRoot
                        } else {
                            BindingAnchor::ExternRoot
                        },
                        visibility: visibility(&item.vis),
                        quality: quality(&item.attrs),
                        replacement_macros: replacement_macros(&item.attrs, &guard, lexical_scope),
                        guard,
                        scope: lexical_scope,
                    },
                );
            }
            Item::Type(item) => {
                let guard = item_guard(&item.attrs, enclosing_guard);
                let (target, kind, anchor) = match item.ty.as_ref() {
                    Type::Path(target) if target.qself.is_none() => (
                        path_text(&target.path),
                        BindingKind::TypeAlias,
                        if target.path.leading_colon.is_some() {
                            BindingAnchor::Absolute
                        } else {
                            BindingAnchor::Lexical
                        },
                    ),
                    _ => (
                        item.ident.to_string(),
                        BindingKind::OpaqueAlias,
                        BindingAnchor::Lexical,
                    ),
                };
                push(
                    &mut bindings,
                    BindingDraft {
                        name: Some(item.ident.to_string()),
                        target,
                        kind,
                        anchor,
                        visibility: visibility(&item.vis),
                        quality: quality(&item.attrs),
                        replacement_macros: replacement_macros(&item.attrs, &guard, lexical_scope),
                        guard,
                        scope: lexical_scope,
                    },
                );
            }
            Item::Enum(item) => {
                local(
                    &mut bindings,
                    item.ident.to_string(),
                    BindingKind::LocalType,
                    &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
                );
                enum_variants(&mut bindings, item, enclosing_guard, lexical_scope);
            }
            Item::Mod(item) => module(&mut bindings, item, enclosing_guard, lexical_scope),
            Item::Struct(item) => {
                let kind = match item.fields {
                    syn::Fields::Named(_) => BindingKind::LocalType,
                    syn::Fields::Unnamed(_) => {
                        BindingKind::LocalConstructor(ConstructorForm::Tuple)
                    }
                    syn::Fields::Unit => BindingKind::LocalConstructor(ConstructorForm::Unit),
                };
                local(
                    &mut bindings,
                    item.ident.to_string(),
                    kind,
                    &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
                );
            }
            Item::Trait(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::TraitAlias(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Union(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Const(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalValue,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Fn(item) => local(
                &mut bindings,
                item.sig.ident.to_string(),
                BindingKind::LocalValue,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Static(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalValue,
                &context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::ForeignMod(item) => {
                foreign(&mut bindings, item, enclosing_guard, lexical_scope);
            }
            _ => {}
        }
    }
    bindings
}

fn context<'a>(
    attributes: &'a [syn::Attribute],
    visibility: &'a syn::Visibility,
    enclosing_guard: &SyntaxGuard,
    scope: &'a [SourceSpan],
) -> BindingContext<'a> {
    BindingContext {
        attributes,
        visibility,
        enclosing_guard: enclosing_guard.clone(),
        scope,
    }
}
