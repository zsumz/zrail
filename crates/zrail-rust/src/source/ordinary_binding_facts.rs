//! Binding facts retain namespace kind, visibility, guards, and absolute anchors.

use syn::{Attribute, Visibility, spanned::Spanned};
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    BindingAnchor, BindingKind, BindingVisibility, ImportBindingFact, ModuleBinding, SyntaxGuard,
    attributes::cfg_guard, fact::source_span, macro_binding_policy::MacroOccurrence,
};

#[derive(Clone)]
pub(super) struct BindingContext<'a> {
    pub(super) attributes: &'a [Attribute],
    pub(super) visibility: &'a Visibility,
    pub(super) enclosing_guard: SyntaxGuard,
    pub(super) scope: &'a [SourceSpan],
}

pub(super) struct BindingDraft<'a> {
    pub(super) name: Option<String>,
    pub(super) target: String,
    pub(super) kind: BindingKind,
    pub(super) anchor: BindingAnchor,
    pub(super) visibility: BindingVisibility,
    pub(super) quality: AnalysisQuality,
    pub(super) replacement_macros: Vec<MacroOccurrence>,
    pub(super) guard: SyntaxGuard,
    pub(super) scope: &'a [SourceSpan],
}

pub(super) fn local(
    bindings: &mut Vec<ImportBindingFact>,
    name: String,
    kind: BindingKind,
    context: &BindingContext<'_>,
) {
    let guard = item_guard(context.attributes, &context.enclosing_guard);
    push(
        bindings,
        BindingDraft {
            name: Some(name.clone()),
            target: name,
            kind,
            anchor: BindingAnchor::Lexical,
            visibility: visibility(context.visibility),
            quality: quality(context.attributes),
            replacement_macros: replacement_macros(context.attributes, &guard, context.scope),
            guard,
            scope: context.scope,
        },
    );
}

pub(super) fn module(
    bindings: &mut Vec<ImportBindingFact>,
    item: &syn::ItemMod,
    enclosing_guard: &SyntaxGuard,
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
        BindingDraft {
            name: Some(name.clone()),
            target: name,
            kind: BindingKind::Module(module),
            anchor: BindingAnchor::Lexical,
            visibility: visibility(&item.vis),
            quality: quality(&item.attrs),
            replacement_macros: replacement_macros(&item.attrs, &guard, scope),
            guard,
            scope,
        },
    );
}

pub(super) fn foreign(
    bindings: &mut Vec<ImportBindingFact>,
    block: &syn::ItemForeignMod,
    enclosing_guard: &SyntaxGuard,
    scope: &[SourceSpan],
) {
    let guard = item_guard(&block.attrs, enclosing_guard);
    let outer_quality = quality(&block.attrs);
    let outer_macros = replacement_macros(&block.attrs, &guard, scope);
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
            &BindingContext {
                attributes,
                visibility,
                enclosing_guard: guard.clone(),
                scope,
            },
        );
        let binding = &mut bindings[start];
        binding.quality_without_macros = binding.quality_without_macros.max(outer_quality);
        binding
            .replacement_macros
            .extend(outer_macros.iter().cloned());
        binding.replacement_macros.sort();
        binding.replacement_macros.dedup();
        binding.quality = if binding.replacement_macros.is_empty() {
            binding.quality_without_macros
        } else {
            AnalysisQuality::Unresolved
        };
    }
}

pub(super) fn push(bindings: &mut Vec<ImportBindingFact>, draft: BindingDraft<'_>) {
    let quality = if draft.replacement_macros.is_empty() {
        draft.quality
    } else {
        AnalysisQuality::Unresolved
    };
    bindings.push(ImportBindingFact {
        name: draft.name,
        target: draft.target,
        kind: draft.kind,
        anchor: draft.anchor,
        visibility: draft.visibility,
        quality,
        quality_without_macros: draft.quality,
        replacement_macros: draft.replacement_macros,
        guard: draft.guard,
        lexical_scope: draft.scope.to_vec(),
    });
}

pub(super) fn quality(_attributes: &[Attribute]) -> AnalysisQuality {
    AnalysisQuality::Exact
}

pub(super) fn replacement_macros(
    attributes: &[Attribute],
    guard: &SyntaxGuard,
    scope: &[SourceSpan],
) -> Vec<MacroOccurrence> {
    let mut occurrences = attributes
        .iter()
        .filter(|attribute| super::macro_expansion::can_replace_item(attribute))
        .flat_map(|attribute| {
            super::macro_expansion::attribute_paths(attribute).map_or_else(
                |()| vec![(attribute.path().clone(), super::SyntaxGuard::Ordinary)],
                |expansions| {
                    expansions
                        .into_iter()
                        .filter(|expansion| {
                            expansion.kind == super::macro_expansion::ExpansionKind::Attribute
                        })
                        .map(|expansion| (expansion.path, expansion.guard))
                        .collect()
                },
            )
        })
        .map(|(path, effect_guard)| {
            MacroOccurrence::new(
                source_span(path.span()),
                &guard.combine(effect_guard),
                scope,
            )
        })
        .collect::<Vec<_>>();
    occurrences.sort();
    occurrences.dedup();
    occurrences
}

pub(super) fn item_guard(attributes: &[Attribute], enclosing: &SyntaxGuard) -> SyntaxGuard {
    enclosing.combine(cfg_guard(attributes))
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
