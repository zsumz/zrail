//! Lexical glob imports remain candidates only inside their actual Rust scope.

use std::collections::BTreeMap;

use syn::{Item, UseTree};

use super::{SyntaxGuard, attributes::is_cfg_test, import_helpers::insert_guard};

pub(super) fn collect<'a>(items: impl Iterator<Item = &'a Item>) -> BTreeMap<String, SyntaxGuard> {
    let mut globs = BTreeMap::new();
    for item in items {
        if let Item::Use(item) = item {
            collect_use(
                &mut globs,
                Vec::new(),
                &item.tree,
                SyntaxGuard::for_test_only(item.attrs.iter().any(is_cfg_test)),
            );
        }
    }
    globs
}

fn collect_use(
    globs: &mut BTreeMap<String, SyntaxGuard>,
    prefix: Vec<String>,
    tree: &UseTree,
    guard: SyntaxGuard,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(globs, nested, &path.tree, guard);
        }
        UseTree::Glob(_) if !prefix.is_empty() => insert_guard(globs, prefix.join("::"), guard),
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(globs, prefix.clone(), tree, guard);
            }
        }
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
    }
}

#[cfg(test)]
#[path = "scoped_globs_test.rs"]
mod scoped_globs_test;
