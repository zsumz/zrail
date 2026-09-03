//! Public use trees are flattened without importing the type namespace.

use syn::{ItemUse, UseTree};

use super::{ModuleSurface, UseBinding, conditional, identifier};

pub(super) fn collect(item: &ItemUse, surface: &mut ModuleSurface) {
    collect_tree(Vec::new(), &item.tree, conditional(&item.attrs), surface);
}

fn collect_tree(
    prefix: Vec<String>,
    tree: &UseTree,
    conditional: bool,
    surface: &mut ModuleSurface,
) {
    match tree {
        UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(identifier(&path.ident));
            collect_tree(prefix, &path.tree, conditional, surface);
        }
        UseTree::Name(name) => insert(prefix, identifier(&name.ident), conditional, surface),
        UseTree::Rename(rename) => {
            let mut target = prefix;
            let name = identifier(&rename.ident);
            if name != "self" {
                target.push(name);
            }
            surface.bindings.push(UseBinding {
                exported: identifier(&rename.rename),
                target,
                conditional,
            });
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_tree(prefix.clone(), tree, conditional, surface);
            }
        }
        UseTree::Glob(_) => {
            surface
                .open
                .get_or_insert_with(|| "external module contains a public glob re-export".into());
        }
    }
}

fn insert(mut target: Vec<String>, name: String, conditional: bool, surface: &mut ModuleSurface) {
    let exported = if name == "self" {
        target.last().cloned().unwrap_or_default()
    } else {
        target.push(name.clone());
        name
    };
    if !exported.is_empty() {
        surface.bindings.push(UseBinding {
            exported,
            target,
            conditional,
        });
    }
}
