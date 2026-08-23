//! Use trees flatten into guarded ordinary binding facts.

use syn::UseTree;
use zrail_core::{AnalysisQuality, SourceSpan};

use super::{
    BindingAnchor, BindingKind, BindingVisibility, ImportBindingFact, SyntaxGuard,
    macro_binding_policy::MacroOccurrence,
    ordinary_binding_facts::{BindingDraft, push, use_target},
};

pub(super) struct UseBindingContext<'a> {
    pub(super) anchor: BindingAnchor,
    pub(super) visibility: BindingVisibility,
    pub(super) quality: AnalysisQuality,
    pub(super) replacement_macros: Vec<MacroOccurrence>,
    pub(super) guard: SyntaxGuard,
    pub(super) scope: &'a [SourceSpan],
}

pub(super) fn collect_use(
    bindings: &mut Vec<ImportBindingFact>,
    prefix: Vec<String>,
    tree: &UseTree,
    context: &UseBindingContext<'_>,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(bindings, nested, &path.tree, context);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                push_binding(
                    bindings,
                    Some(alias.clone()),
                    &prefix,
                    BindingKind::Import,
                    context,
                );
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            push_binding(
                bindings,
                Some(name.ident.to_string()),
                &target,
                BindingKind::Import,
                context,
            );
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            if !target.is_empty() {
                push_binding(
                    bindings,
                    Some(rename.rename.to_string()),
                    &target,
                    BindingKind::Import,
                    context,
                );
            }
        }
        UseTree::Glob(_) if !prefix.is_empty() => {
            push_binding(bindings, None, &prefix, BindingKind::Glob, context);
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(bindings, prefix.clone(), tree, context);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn push_binding(
    bindings: &mut Vec<ImportBindingFact>,
    name: Option<String>,
    target: &[String],
    kind: BindingKind,
    context: &UseBindingContext<'_>,
) {
    let quality = if kind == BindingKind::Glob {
        context.quality.max(AnalysisQuality::Conservative)
    } else {
        context.quality
    };
    push(
        bindings,
        BindingDraft {
            name,
            target: use_target(target),
            kind,
            anchor: context.anchor,
            visibility: context.visibility.clone(),
            quality,
            replacement_macros: context.replacement_macros.clone(),
            guard: context.guard,
            scope: context.scope,
        },
    );
}
