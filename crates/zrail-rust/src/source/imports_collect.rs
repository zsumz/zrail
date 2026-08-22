//! Exact top-level imports retain conditional and re-export authority.

use syn::UseTree;

use super::imports::ImportMap;

pub(super) fn collect_use(
    imports: &mut ImportMap,
    prefix: Vec<String>,
    tree: &UseTree,
    conditional: bool,
    re_export: bool,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(imports, nested, &path.tree, conditional, re_export);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                imports.aliases.insert(alias.clone(), prefix.join("::"));
                if conditional {
                    imports.unresolved.insert(alias.clone());
                }
                if re_export {
                    imports.re_exports.insert(alias.clone());
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
                re_export,
            );
        }
        UseTree::Glob(_) => {
            let glob = prefix.join("::");
            if re_export {
                imports.re_export_globs.insert(glob.clone());
            }
            imports.globs.push(glob);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use(imports, prefix.clone(), item, conditional, re_export);
            }
        }
    }
}

fn insert_alias(
    imports: &mut ImportMap,
    alias: String,
    target: &[String],
    conditional: bool,
    re_export: bool,
) {
    imports.aliases.insert(alias.clone(), target.join("::"));
    if conditional {
        imports.unresolved.insert(alias.clone());
    }
    if re_export {
        imports.re_exports.insert(alias);
    }
}
