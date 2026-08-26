//! Exact top-level imports retain conditional and re-export authority.

use syn::UseTree;

use super::{
    SyntaxGuard,
    import_helpers::{insert_guard, insert_primary_alias},
    imports::ImportMap,
};

pub(super) fn collect_use(
    imports: &mut ImportMap,
    prefix: Vec<String>,
    tree: &UseTree,
    conditional: bool,
    guard: &SyntaxGuard,
    re_export: bool,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(imports, nested, &path.tree, conditional, guard, re_export);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                insert_primary_alias(
                    &mut imports.aliases,
                    &mut imports.alias_guards,
                    &mut imports.unresolved,
                    alias,
                    prefix.join("::"),
                    conditional,
                    guard,
                );
                if re_export {
                    insert_guard(&mut imports.re_exports, alias.clone(), guard);
                }
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            insert_alias(
                imports,
                name.ident.to_string(),
                &target,
                conditional,
                guard,
                re_export,
            );
        }
        UseTree::Rename(rename) if rename.ident == "self" => {
            if !prefix.is_empty() {
                insert_alias(
                    imports,
                    rename.rename.to_string(),
                    &prefix,
                    conditional,
                    guard,
                    re_export,
                );
            }
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            target.push(rename.ident.to_string());
            insert_alias(
                imports,
                rename.rename.to_string(),
                &target,
                conditional,
                guard,
                re_export,
            );
        }
        UseTree::Glob(_) => {
            let glob = prefix.join("::");
            if re_export {
                insert_guard(&mut imports.re_export_globs, glob.clone(), guard);
            }
            insert_guard(&mut imports.globs, glob, guard);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use(imports, prefix.clone(), item, conditional, guard, re_export);
            }
        }
    }
}

fn insert_alias(
    imports: &mut ImportMap,
    alias: String,
    target: &[String],
    conditional: bool,
    guard: &SyntaxGuard,
    re_export: bool,
) {
    insert_primary_alias(
        &mut imports.aliases,
        &mut imports.alias_guards,
        &mut imports.unresolved,
        &alias,
        target.join("::"),
        conditional,
        guard,
    );
    if re_export {
        insert_guard(&mut imports.re_exports, alias, guard);
    }
}
