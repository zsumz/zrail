//! Ordinary type-path bindings retain every guarded declaration in one lexical scope.

use syn::{Item, Type, UseTree};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    BindingAnchor, BindingKind, ImportBindingFact, SyntaxGuard,
    attributes::is_cfg_test,
    ordinary_binding_facts::{
        BindingContext, foreign, item_guard, local, module, path_text, push, quality, use_target,
        visibility,
    },
};

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    enclosing_guard: SyntaxGuard,
    lexical_scope: &[SourceSpan],
) -> Vec<ImportBindingFact> {
    let mut bindings = Vec::new();
    for item in items {
        match item {
            Item::Use(item) => {
                let guard = enclosing_guard.combine(SyntaxGuard::for_test_only(
                    item.attrs.iter().any(is_cfg_test),
                ));
                collect_use(
                    &mut bindings,
                    Vec::new(),
                    &item.tree,
                    if item.leading_colon.is_some() {
                        BindingAnchor::Absolute
                    } else {
                        BindingAnchor::UsePath
                    },
                    visibility(&item.vis),
                    quality(&item.attrs, guard),
                    guard,
                    lexical_scope,
                );
            }
            Item::ExternCrate(item) => {
                let guard = item_guard(&item.attrs, enclosing_guard);
                let name = item
                    .rename
                    .as_ref()
                    .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                push(
                    &mut bindings,
                    Some(name),
                    item.ident.to_string(),
                    BindingKind::Import,
                    if item.ident == "self" {
                        BindingAnchor::CrateRoot
                    } else {
                        BindingAnchor::ExternRoot
                    },
                    visibility(&item.vis),
                    quality(&item.attrs, guard),
                    guard,
                    lexical_scope,
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
                    Some(item.ident.to_string()),
                    target,
                    kind,
                    anchor,
                    visibility(&item.vis),
                    quality(&item.attrs, guard),
                    guard,
                    lexical_scope,
                );
            }
            Item::Enum(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Mod(item) => module(&mut bindings, item, enclosing_guard, lexical_scope),
            Item::Struct(item) => {
                let kind = if matches!(item.fields, syn::Fields::Unnamed(_) | syn::Fields::Unit) {
                    BindingKind::LocalConstructor
                } else {
                    BindingKind::LocalType
                };
                local(
                    &mut bindings,
                    item.ident.to_string(),
                    kind,
                    context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
                );
            }
            Item::Trait(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::TraitAlias(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Union(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalType,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Const(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalValue,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Fn(item) => local(
                &mut bindings,
                item.sig.ident.to_string(),
                BindingKind::LocalValue,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::Static(item) => local(
                &mut bindings,
                item.ident.to_string(),
                BindingKind::LocalValue,
                context(&item.attrs, &item.vis, enclosing_guard, lexical_scope),
            ),
            Item::ForeignMod(item) => {
                foreign(&mut bindings, item, enclosing_guard, lexical_scope);
            }
            _ => {}
        }
    }
    bindings
}

fn collect_use(
    bindings: &mut Vec<ImportBindingFact>,
    prefix: Vec<String>,
    tree: &UseTree,
    anchor: BindingAnchor,
    visibility: super::BindingVisibility,
    quality: AnalysisQuality,
    guard: SyntaxGuard,
    scope: &[SourceSpan],
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(
                bindings, nested, &path.tree, anchor, visibility, quality, guard, scope,
            );
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                push(
                    bindings,
                    Some(alias.clone()),
                    use_target(&prefix),
                    BindingKind::Import,
                    anchor,
                    visibility.clone(),
                    quality,
                    guard,
                    scope,
                );
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            push(
                bindings,
                Some(name.ident.to_string()),
                use_target(&target),
                BindingKind::Import,
                anchor,
                visibility.clone(),
                quality,
                guard,
                scope,
            );
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            if !target.is_empty() {
                push(
                    bindings,
                    Some(rename.rename.to_string()),
                    use_target(&target),
                    BindingKind::Import,
                    anchor,
                    visibility.clone(),
                    quality,
                    guard,
                    scope,
                );
            }
        }
        UseTree::Glob(_) if !prefix.is_empty() => {
            push(
                bindings,
                None,
                use_target(&prefix),
                BindingKind::Glob,
                anchor,
                visibility.clone(),
                quality.max(AnalysisQuality::Conservative),
                guard,
                scope,
            );
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(
                    bindings,
                    prefix.clone(),
                    tree,
                    anchor,
                    visibility.clone(),
                    quality,
                    guard,
                    scope,
                );
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn context<'a>(
    attributes: &'a [syn::Attribute],
    visibility: &'a syn::Visibility,
    enclosing_guard: SyntaxGuard,
    scope: &'a [SourceSpan],
) -> BindingContext<'a> {
    BindingContext {
        attributes,
        visibility,
        enclosing_guard,
        scope,
    }
}
