//! Binding facts retain namespace kind, visibility, guards, and absolute anchors.

use syn::{Attribute, Visibility};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    BindingAnchor, BindingKind, BindingVisibility, ImportBindingFact, ModuleBinding, SyntaxGuard,
    attributes::is_cfg_test, fact::source_span,
};

#[derive(Clone, Copy)]
pub(super) struct BindingContext<'a> {
    pub(super) attributes: &'a [Attribute],
    pub(super) visibility: &'a Visibility,
    pub(super) enclosing_guard: SyntaxGuard,
    pub(super) scope: &'a [SourceSpan],
}

pub(super) fn local(
    bindings: &mut Vec<ImportBindingFact>,
    name: String,
    kind: BindingKind,
    context: BindingContext<'_>,
) {
    let guard = item_guard(context.attributes, context.enclosing_guard);
    push(
        bindings,
        Some(name.clone()),
        name,
        kind,
        BindingAnchor::Lexical,
        visibility(context.visibility),
        quality(context.attributes, guard),
        guard,
        context.scope,
    );
}

pub(super) fn module(
    bindings: &mut Vec<ImportBindingFact>,
    item: &syn::ItemMod,
    enclosing_guard: SyntaxGuard,
    scope: &[SourceSpan],
) {
    let guard = item_guard(&item.attrs, enclosing_guard);
    let span = source_span(item.ident.span());
    let module = if item.content.is_some() {
        ModuleBinding::Inline(span)
    } else {
        ModuleBinding::External(span)
    };
    let name = item.ident.to_string();
    push(
        bindings,
        Some(name.clone()),
        name,
        BindingKind::Module(module),
        BindingAnchor::Lexical,
        visibility(&item.vis),
        quality(&item.attrs, guard),
        guard,
        scope,
    );
}

pub(super) fn foreign(
    bindings: &mut Vec<ImportBindingFact>,
    block: &syn::ItemForeignMod,
    enclosing_guard: SyntaxGuard,
    scope: &[SourceSpan],
) {
    let guard = item_guard(&block.attrs, enclosing_guard);
    let outer_quality = quality(&block.attrs, guard);
    for item in &block.items {
        let (name, kind, attributes, visibility) = match item {
            syn::ForeignItem::Fn(item) => (
                item.sig.ident.to_string(),
                BindingKind::LocalValue,
                item.attrs.as_slice(),
                &item.vis,
            ),
            syn::ForeignItem::Static(item) => (
                item.ident.to_string(),
                BindingKind::LocalValue,
                item.attrs.as_slice(),
                &item.vis,
            ),
            syn::ForeignItem::Type(item) => (
                item.ident.to_string(),
                BindingKind::LocalType,
                item.attrs.as_slice(),
                &item.vis,
            ),
            _ => continue,
        };
        let start = bindings.len();
        local(
            bindings,
            name,
            kind,
            BindingContext {
                attributes,
                visibility,
                enclosing_guard: guard,
                scope,
            },
        );
        bindings[start].quality = bindings[start].quality.max(outer_quality);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push(
    bindings: &mut Vec<ImportBindingFact>,
    name: Option<String>,
    target: String,
    kind: BindingKind,
    anchor: BindingAnchor,
    visibility: BindingVisibility,
    quality: AnalysisQuality,
    guard: SyntaxGuard,
    lexical_scope: &[SourceSpan],
) {
    bindings.push(ImportBindingFact {
        name,
        target,
        kind,
        anchor,
        visibility,
        quality,
        guard,
        lexical_scope: lexical_scope.to_vec(),
    });
}

pub(super) fn quality(attributes: &[Attribute], guard: SyntaxGuard) -> AnalysisQuality {
    if attributes
        .iter()
        .any(super::macro_expansion::can_replace_item)
        || (guard == SyntaxGuard::Ordinary && super::scoped_imports::conditional(attributes))
    {
        AnalysisQuality::Unresolved
    } else {
        AnalysisQuality::Exact
    }
}

pub(super) fn item_guard(attributes: &[Attribute], enclosing: SyntaxGuard) -> SyntaxGuard {
    enclosing.combine(SyntaxGuard::for_test_only(
        attributes.iter().any(is_cfg_test),
    ))
}

pub(super) fn visibility(value: &Visibility) -> BindingVisibility {
    match value {
        Visibility::Public(_) => BindingVisibility::Public,
        Visibility::Inherited => BindingVisibility::Private,
        Visibility::Restricted(restricted) => BindingVisibility::Restricted(
            restricted
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        ),
    }
}

pub(super) fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(super) fn use_target(segments: &[String]) -> String {
    segments.join("::")
}
