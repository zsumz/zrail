//! Ordinary type-path bindings retain every guarded declaration in one lexical scope.

use syn::{Item, Type, UseTree};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{ImportBindingFact, SyntaxGuard, attributes::is_cfg_test};

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
                    quality(&item.attrs, guard),
                    guard,
                    lexical_scope,
                );
            }
            Item::Type(item) => {
                if let Type::Path(target) = item.ty.as_ref()
                    && target.qself.is_none()
                {
                    let guard = item_guard(&item.attrs, enclosing_guard);
                    push(
                        &mut bindings,
                        Some(item.ident.to_string()),
                        path_text(&target.path),
                        quality(&item.attrs, guard),
                        guard,
                        lexical_scope,
                    );
                }
            }
            Item::Enum(item) => local(
                &mut bindings,
                item.ident.to_string(),
                &item.attrs,
                enclosing_guard,
                lexical_scope,
            ),
            Item::Mod(item) => local(
                &mut bindings,
                item.ident.to_string(),
                &item.attrs,
                enclosing_guard,
                lexical_scope,
            ),
            Item::Struct(item) => local(
                &mut bindings,
                item.ident.to_string(),
                &item.attrs,
                enclosing_guard,
                lexical_scope,
            ),
            Item::Trait(item) => local(
                &mut bindings,
                item.ident.to_string(),
                &item.attrs,
                enclosing_guard,
                lexical_scope,
            ),
            Item::Union(item) => local(
                &mut bindings,
                item.ident.to_string(),
                &item.attrs,
                enclosing_guard,
                lexical_scope,
            ),
            _ => {}
        }
    }
    bindings
}

fn collect_use(
    bindings: &mut Vec<ImportBindingFact>,
    prefix: Vec<String>,
    tree: &UseTree,
    quality: AnalysisQuality,
    guard: SyntaxGuard,
    scope: &[SourceSpan],
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(bindings, nested, &path.tree, quality, guard, scope);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                push(
                    bindings,
                    Some(alias.clone()),
                    prefix.join("::"),
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
                target.join("::"),
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
                    target.join("::"),
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
                prefix.join("::"),
                quality.max(AnalysisQuality::Conservative),
                guard,
                scope,
            );
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(bindings, prefix.clone(), tree, quality, guard, scope);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn local(
    bindings: &mut Vec<ImportBindingFact>,
    name: String,
    attributes: &[syn::Attribute],
    guard: SyntaxGuard,
    scope: &[SourceSpan],
) {
    let guard = item_guard(attributes, guard);
    push(
        bindings,
        Some(name.clone()),
        name,
        quality(attributes, guard),
        guard,
        scope,
    );
}

fn push(
    bindings: &mut Vec<ImportBindingFact>,
    name: Option<String>,
    target: String,
    quality: AnalysisQuality,
    guard: SyntaxGuard,
    lexical_scope: &[SourceSpan],
) {
    bindings.push(ImportBindingFact {
        name,
        target,
        quality,
        guard,
        lexical_scope: lexical_scope.to_vec(),
    });
}

fn quality(attributes: &[syn::Attribute], guard: SyntaxGuard) -> AnalysisQuality {
    if guard == SyntaxGuard::Ordinary && super::scoped_imports::conditional(attributes) {
        AnalysisQuality::Unresolved
    } else {
        AnalysisQuality::Exact
    }
}

fn item_guard(attributes: &[syn::Attribute], enclosing: SyntaxGuard) -> SyntaxGuard {
    enclosing.combine(SyntaxGuard::for_test_only(
        attributes.iter().any(is_cfg_test),
    ))
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
