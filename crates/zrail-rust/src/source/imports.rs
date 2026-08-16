//! Import aliases and glob roots for syntax-derived path resolution.

use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, UseTree};
use zrail_core::AnalysisQuality;

#[derive(Clone, Debug, Default)]
pub(super) struct ImportMap {
    aliases: BTreeMap<String, String>,
    call_aliases: BTreeMap<String, BTreeSet<String>>,
    unresolved: BTreeSet<String>,
    globs: Vec<String>,
}

impl ImportMap {
    pub(super) fn from_file(file: &syn::File) -> Self {
        let mut imports = Self::default();
        for item in &file.items {
            match item {
                Item::Use(item) => collect_use(
                    &mut imports,
                    Vec::new(),
                    &item.tree,
                    super::scoped_imports::conditional(&item.attrs),
                ),
                Item::ExternCrate(item) => {
                    let alias = item
                        .rename
                        .as_ref()
                        .map_or_else(|| item.ident.to_string(), |(_, name)| name.to_string());
                    imports
                        .aliases
                        .insert(alias.clone(), item.ident.to_string());
                    if super::scoped_imports::conditional(&item.attrs) {
                        imports.unresolved.insert(alias);
                    }
                }
                _ => {}
            }
        }
        let candidates = super::import_candidates::collect(file);
        imports.call_aliases = candidates.aliases;
        imports.globs.extend(candidates.globs);
        imports.normalize_aliases();
        super::import_candidates::normalize(&mut imports.call_aliases, &imports.aliases);
        imports.globs.sort();
        imports.globs.dedup();
        imports
    }

    pub(super) fn resolve(&self, path: &syn::Path) -> (String, AnalysisQuality) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return (String::new(), AnalysisQuality::Unresolved);
        };
        if let Some(prefix) = self.aliases.get(first) {
            let mut resolved = prefix.clone();
            if segments.len() > 1 {
                resolved.push_str("::");
                resolved.push_str(&segments[1..].join("::"));
            }
            let quality = if self.unresolved.contains(first) {
                AnalysisQuality::Unresolved
            } else {
                AnalysisQuality::Exact
            };
            return (resolved, quality);
        }
        (segments.join("::"), AnalysisQuality::Exact)
    }

    pub(super) fn declared_paths(&self) -> Vec<(&str, AnalysisQuality)> {
        let mut paths = self
            .aliases
            .iter()
            .map(|(alias, path)| {
                let quality = if self.unresolved.contains(alias) {
                    AnalysisQuality::Unresolved
                } else {
                    AnalysisQuality::Exact
                };
                (path.as_str(), quality)
            })
            .collect::<Vec<_>>();
        paths.sort_unstable();
        paths.dedup();
        paths
    }

    pub(super) fn globs(&self) -> &[String] {
        &self.globs
    }

    pub(super) fn call_candidates(&self, path: &syn::Path) -> Vec<String> {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return Vec::new();
        };
        let remainder = &segments[1..];
        let mut candidates = self
            .call_aliases
            .get(first)
            .into_iter()
            .flatten()
            .map(|prefix| join_path(prefix.clone(), remainder))
            .collect::<Vec<_>>();
        let syntactic = segments.join("::");
        candidates.extend(self.globs.iter().map(|glob| format!("{glob}::{syntactic}")));
        candidates.sort();
        candidates.dedup();
        candidates
    }

    fn normalize_aliases(&mut self) {
        let raw = self.aliases.clone();
        let mut cache = BTreeMap::new();
        for alias in raw.keys() {
            let mut visiting = BTreeSet::new();
            match super::import_aliases::expand_alias(alias, &raw, &mut visiting, &mut cache, 0) {
                Some(target) => {
                    self.aliases.insert(alias.clone(), target);
                }
                None => {
                    self.unresolved.insert(alias.clone());
                }
            }
        }
    }
}

fn collect_use(imports: &mut ImportMap, prefix: Vec<String>, tree: &UseTree, conditional: bool) {
    match tree {
        UseTree::Path(path) => {
            let mut nested = prefix;
            nested.push(path.ident.to_string());
            collect_use(imports, nested, &path.tree, conditional);
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(alias) = prefix.last() {
                imports.aliases.insert(alias.clone(), prefix.join("::"));
                if conditional {
                    imports.unresolved.insert(alias.clone());
                }
            }
        }
        UseTree::Name(name) => {
            let mut target = prefix;
            target.push(name.ident.to_string());
            let alias = name.ident.to_string();
            imports.aliases.insert(alias.clone(), target.join("::"));
            if conditional {
                imports.unresolved.insert(alias);
            }
        }
        UseTree::Rename(rename) if rename.ident == "self" => {
            if !prefix.is_empty() {
                let alias = rename.rename.to_string();
                imports.aliases.insert(alias.clone(), prefix.join("::"));
                if conditional {
                    imports.unresolved.insert(alias);
                }
            }
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            target.push(rename.ident.to_string());
            let alias = rename.rename.to_string();
            imports.aliases.insert(alias.clone(), target.join("::"));
            if conditional {
                imports.unresolved.insert(alias);
            }
        }
        UseTree::Glob(_) => imports.globs.push(prefix.join("::")),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use(imports, prefix.clone(), item, conditional);
            }
        }
    }
}

fn join_path(mut prefix: String, remainder: &[String]) -> String {
    if !remainder.is_empty() {
        prefix.push_str("::");
        prefix.push_str(&remainder.join("::"));
    }
    prefix
}

#[cfg(test)]
#[path = "imports_test.rs"]
mod imports_test;
