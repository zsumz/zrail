//! Written glob imports remain visible independently from name resolution.

use syn::{Item, UseTree, spanned::Spanned};
use zrail_core::SourceSpan;

use super::{BindingVisibility, SyntaxGuard, fact::source_span};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobImportFact {
    pub(crate) target: String,
    pub(crate) visibility: BindingVisibility,
    pub(crate) span: SourceSpan,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    enclosing_guard: &SyntaxGuard,
    lexical_scope: &[SourceSpan],
) -> Vec<GlobImportFact> {
    let mut facts = Vec::new();
    for item in items {
        let Item::Use(item) = item else { continue };
        let guard = super::ordinary_binding_facts::item_guard(&item.attrs, enclosing_guard);
        let visibility = super::ordinary_binding_facts::visibility(&item.vis);
        let mut prefix = item
            .leading_colon
            .is_some()
            .then(String::new)
            .into_iter()
            .collect();
        collect_tree(
            &item.tree,
            &mut prefix,
            &visibility,
            &guard,
            lexical_scope,
            &mut facts,
        );
    }
    facts
}

fn collect_tree(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    visibility: &BindingVisibility,
    guard: &SyntaxGuard,
    lexical_scope: &[SourceSpan],
    facts: &mut Vec<GlobImportFact>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_tree(&path.tree, prefix, visibility, guard, lexical_scope, facts);
            prefix.pop();
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_tree(nested, prefix, visibility, guard, lexical_scope, facts);
            }
        }
        UseTree::Glob(glob) => facts.push(GlobImportFact {
            target: prefix.join("::"),
            visibility: visibility.clone(),
            span: source_span(glob.star_token.span()),
            guard: guard.clone(),
            lexical_scope: lexical_scope.to_vec(),
        }),
        UseTree::Name(_) | UseTree::Rename(_) => {}
    }
}

#[cfg(test)]
#[path = "glob_imports_test.rs"]
mod glob_imports_test;
