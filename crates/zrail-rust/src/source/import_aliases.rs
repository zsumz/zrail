//! Bounded expansion of exact top-level Rust import aliases.

use std::collections::{BTreeMap, BTreeSet};

const MAX_EXPANDED_ALIAS_BYTES: usize = 1_024;
const MAX_ALIAS_HOPS: usize = 128;

pub(super) fn expand_alias(
    alias: &str,
    aliases: &BTreeMap<String, String>,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, Option<String>>,
    depth: usize,
) -> Option<String> {
    if let Some(cached) = cache.get(alias) {
        return cached.clone();
    }
    if depth == MAX_ALIAS_HOPS || !visiting.insert(alias.to_owned()) {
        return None;
    }
    let expanded = (|| {
        let target = aliases.get(alias)?.clone();
        let (first, suffix) = split_root(&target);
        let mut prefix = if aliases.contains_key(first) && first != alias {
            expand_alias(first, aliases, visiting, cache, depth + 1)?
        } else {
            first.into()
        };
        if prefix.len().saturating_add(suffix.len()) > MAX_EXPANDED_ALIAS_BYTES {
            None
        } else {
            prefix.push_str(suffix);
            Some(prefix)
        }
    })();
    visiting.remove(alias);
    cache.insert(alias.into(), expanded.clone());
    expanded
}

fn split_root(path: &str) -> (&str, &str) {
    path.find("::").map_or((path, ""), |separator| {
        (&path[..separator], &path[separator..])
    })
}
