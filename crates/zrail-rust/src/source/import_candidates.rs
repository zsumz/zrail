//! Conservative file-wide call aliases for scope-sensitive Rust syntax.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Type, UseTree, visit::Visit as _};

#[derive(Default)]
pub(super) struct CallCandidates {
    pub(super) aliases: BTreeMap<String, BTreeSet<String>>,
    pub(super) globs: Vec<String>,
}

pub(super) fn collect(file: &syn::File) -> CallCandidates {
    let mut candidates = CallCandidates::default();
    candidates.visit_file(file);
    candidates
}

pub(super) fn normalize(
    candidates: &mut BTreeMap<String, BTreeSet<String>>,
    aliases: &BTreeMap<String, String>,
) {
    for targets in candidates.values_mut() {
        *targets = targets
            .iter()
            .map(|target| expand_exact_prefix(target, aliases))
            .collect();
    }
}

impl<'ast> syn::visit::Visit<'ast> for CallCandidates {
    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let alias = item
            .rename
            .as_ref()
            .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
        self.aliases
            .entry(alias)
            .or_default()
            .insert(item.ident.to_string());
        syn::visit::visit_item_extern_crate(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        collect_use(self, Vec::new(), &item.tree);
        syn::visit::visit_item_use(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if let Type::Path(target) = item.ty.as_ref()
            && target.qself.is_none()
        {
            self.aliases
                .entry(item.ident.to_string())
                .or_default()
                .insert(path_text(&target.path));
        }
        syn::visit::visit_item_type(self, item);
    }
}

fn collect_use(candidates: &mut CallCandidates, prefix: Vec<String>, tree: &UseTree) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(candidates, nested, &path.tree);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                insert_alias(candidates, alias, &prefix);
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            insert_alias(candidates, &name.ident.to_string(), &target);
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            insert_alias(candidates, &rename.rename.to_string(), &target);
        }
        UseTree::Glob(_) => candidates.globs.push(prefix.join("::")),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use(candidates, prefix.clone(), item);
            }
        }
    }
}

fn insert_alias(candidates: &mut CallCandidates, alias: &str, target: &[String]) {
    if !target.is_empty() {
        candidates
            .aliases
            .entry(alias.to_owned())
            .or_default()
            .insert(target.join("::"));
    }
}

fn expand_exact_prefix(target: &str, aliases: &BTreeMap<String, String>) -> String {
    let mut segments = target.split("::");
    let first = segments.next().unwrap_or_default();
    let remainder = segments.collect::<Vec<_>>();
    aliases.get(first).map_or_else(
        || target.to_owned(),
        |prefix| {
            if remainder.is_empty() {
                prefix.clone()
            } else {
                format!("{prefix}::{}", remainder.join("::"))
            }
        },
    )
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
