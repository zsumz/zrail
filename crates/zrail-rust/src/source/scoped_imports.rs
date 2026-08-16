//! Lexical item imports resolve macro paths only inside their actual Rust scope.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, UseTree};
use zrail_core::AnalysisQuality;

const MAX_EXPANDED_ALIAS_BYTES: usize = 1_024;
const MAX_ALIAS_HOPS: usize = 128;

#[derive(Clone, Debug)]
pub(super) struct ScopedAlias {
    pub(super) target: String,
    pub(super) quality: AnalysisQuality,
    pub(super) local_module: bool,
}

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    resolve_outer: impl Fn(&str) -> ScopedAlias,
) -> BTreeMap<String, ScopedAlias> {
    let mut raw = BTreeMap::new();
    let mut modules = BTreeMap::new();
    for item in items {
        match item {
            Item::Use(item) => {
                collect_use(&mut raw, Vec::new(), &item.tree, conditional(&item.attrs));
            }
            Item::ExternCrate(item) => {
                let alias = item
                    .rename
                    .as_ref()
                    .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                raw.insert(
                    alias,
                    (item.ident.to_string(), conditional(&item.attrs), false),
                );
            }
            Item::Mod(module) => {
                let guarded = conditional(&module.attrs);
                modules
                    .entry(module.ident.to_string())
                    .and_modify(|conditional| *conditional |= guarded)
                    .or_insert(guarded);
            }
            Item::Macro(item) => {
                if let Some(name) = &item.ident {
                    let name = name.to_string();
                    raw.insert(name.clone(), (name, true, false));
                }
            }
            _ => {}
        }
    }
    for (module, conditional) in modules {
        raw.insert(module.clone(), (module, conditional, true));
    }
    let mut cache = BTreeMap::new();
    raw.keys()
        .map(|alias| {
            let mut visiting = BTreeSet::new();
            let resolved = expand(alias, &raw, &resolve_outer, &mut visiting, &mut cache, 0)
                .unwrap_or_else(|| unresolved(alias));
            (alias.clone(), resolved)
        })
        .collect()
}

fn expand(
    alias: &str,
    raw: &BTreeMap<String, (String, bool, bool)>,
    resolve_outer: &impl Fn(&str) -> ScopedAlias,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, Option<ScopedAlias>>,
    depth: usize,
) -> Option<ScopedAlias> {
    if let Some(cached) = cache.get(alias) {
        return cached.clone();
    }
    if depth == MAX_ALIAS_HOPS || !visiting.insert(alias.to_owned()) {
        return None;
    }
    let resolved = (|| {
        let (target, conditional, local_module) = raw.get(alias)?;
        let (first, suffix) = split_root(target);
        let mut resolution = if raw.contains_key(first) && first != alias {
            expand(first, raw, resolve_outer, visiting, cache, depth + 1)?
        } else {
            resolve_outer(first)
        };
        if resolution.target.len().saturating_add(suffix.len()) > MAX_EXPANDED_ALIAS_BYTES {
            resolution = unresolved(alias);
        } else {
            resolution.target.push_str(suffix);
        }
        if *conditional {
            resolution.quality = AnalysisQuality::Unresolved;
        }
        resolution.local_module |= *local_module;
        Some(resolution)
    })();
    visiting.remove(alias);
    cache.insert(alias.into(), resolved.clone());
    resolved
}

fn unresolved(alias: &str) -> ScopedAlias {
    ScopedAlias {
        target: alias.into(),
        quality: AnalysisQuality::Unresolved,
        local_module: false,
    }
}

fn collect_use(
    raw: &mut BTreeMap<String, (String, bool, bool)>,
    prefix: Vec<String>,
    tree: &UseTree,
    conditional: bool,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(raw, nested, &path.tree, conditional);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                raw.insert(alias.clone(), (prefix.join("::"), conditional, false));
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            raw.insert(
                name.ident.to_string(),
                (target.join("::"), conditional, false),
            );
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            if !target.is_empty() {
                raw.insert(
                    rename.rename.to_string(),
                    (target.join("::"), conditional, false),
                );
            }
        }
        UseTree::Glob(_) => {}
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(raw, prefix.clone(), tree, conditional);
            }
        }
    }
}

pub(super) fn conditional(attributes: &[syn::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
}

fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}

#[cfg(test)]
#[path = "scoped_imports_test.rs"]
mod scoped_imports_test;
