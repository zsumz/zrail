//! Lexical item imports resolve macro paths only inside their actual Rust scope.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, UseTree};
use zrail_core::AnalysisQuality;

use super::{SyntaxGuard, attributes::is_cfg_test};

const MAX_EXPANDED_ALIAS_BYTES: usize = 1_024;
const MAX_ALIAS_HOPS: usize = 128;

#[derive(Clone, Debug)]
pub(super) struct ScopedAlias {
    pub(super) target: String,
    pub(super) quality: AnalysisQuality,
    pub(super) local_module: bool,
    pub(super) guard: SyntaxGuard,
}

type RawAlias = (String, bool, bool, SyntaxGuard);

pub(super) fn collect<'a>(
    items: impl Iterator<Item = &'a Item>,
    resolve_outer: impl Fn(&str) -> ScopedAlias,
) -> BTreeMap<String, ScopedAlias> {
    let mut raw = BTreeMap::new();
    for item in items {
        match item {
            Item::Use(item) => {
                let guard = SyntaxGuard::for_test_only(item.attrs.iter().any(is_cfg_test));
                collect_use(
                    &mut raw,
                    Vec::new(),
                    &item.tree,
                    conditional(&item.attrs),
                    guard,
                );
            }
            Item::ExternCrate(item) => {
                let alias = item
                    .rename
                    .as_ref()
                    .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                insert_raw(
                    &mut raw,
                    alias,
                    (
                        item.ident.to_string(),
                        conditional(&item.attrs),
                        false,
                        SyntaxGuard::for_test_only(item.attrs.iter().any(is_cfg_test)),
                    ),
                );
            }
            Item::Mod(module) => {
                let guarded = conditional(&module.attrs);
                let guard = SyntaxGuard::for_test_only(module.attrs.iter().any(is_cfg_test));
                let name = module.ident.to_string();
                insert_raw(&mut raw, name.clone(), (name, guarded, true, guard));
            }
            Item::Macro(item) => {
                if let Some(name) = &item.ident {
                    let name = name.to_string();
                    insert_raw(
                        &mut raw,
                        name.clone(),
                        (
                            name,
                            true,
                            true,
                            SyntaxGuard::for_test_only(item.attrs.iter().any(is_cfg_test)),
                        ),
                    );
                }
            }
            _ => {}
        }
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
    raw: &BTreeMap<String, RawAlias>,
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
        let (target, conditional, local_module, guard) = raw.get(alias)?;
        let (first, suffix) = split_root(target);
        let mut resolution = if first == alias {
            ScopedAlias {
                target: first.into(),
                quality: AnalysisQuality::Exact,
                local_module: false,
                guard: SyntaxGuard::Ordinary,
            }
        } else if raw.contains_key(first) {
            expand(first, raw, resolve_outer, visiting, cache, depth + 1)?
        } else {
            resolve_outer(first)
        };
        if resolution.target.len().saturating_add(suffix.len()) > MAX_EXPANDED_ALIAS_BYTES {
            resolution = unresolved(alias);
        } else {
            resolution.target.push_str(suffix);
        }
        resolution.guard = resolution.guard.combine(*guard);
        if *conditional && *guard == SyntaxGuard::Ordinary {
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
        guard: SyntaxGuard::Ordinary,
    }
}

fn collect_use(
    raw: &mut BTreeMap<String, RawAlias>,
    prefix: Vec<String>,
    tree: &UseTree,
    conditional: bool,
    guard: SyntaxGuard,
) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(raw, nested, &path.tree, conditional, guard);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                insert_raw(
                    raw,
                    alias.clone(),
                    (prefix.join("::"), conditional, false, guard),
                );
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            insert_raw(
                raw,
                name.ident.to_string(),
                (target.join("::"), conditional, false, guard),
            );
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            if rename.ident != "self" {
                target.push(rename.ident.to_string());
            }
            if !target.is_empty() {
                insert_raw(
                    raw,
                    rename.rename.to_string(),
                    (target.join("::"), conditional, false, guard),
                );
            }
        }
        UseTree::Glob(_) => {}
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use(raw, prefix.clone(), tree, conditional, guard);
            }
        }
    }
}

fn insert_raw(raw: &mut BTreeMap<String, RawAlias>, alias: String, value: RawAlias) {
    let Some(existing) = raw.get_mut(&alias) else {
        raw.insert(alias, value);
        return;
    };
    match (existing.3, value.3) {
        (SyntaxGuard::TestOnly, SyntaxGuard::Ordinary) => *existing = value,
        (SyntaxGuard::Ordinary, SyntaxGuard::TestOnly) => {}
        _ if existing.0 != value.0 => existing.1 = true,
        _ => existing.1 |= value.1,
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
