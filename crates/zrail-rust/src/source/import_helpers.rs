//! Small deterministic helpers shared by guarded import projections.

use std::collections::{BTreeMap, BTreeSet};

use super::SyntaxGuard;

pub(super) fn insert_guard(
    values: &mut BTreeMap<String, SyntaxGuard>,
    path: String,
    guard: SyntaxGuard,
) {
    values
        .entry(path)
        .and_modify(|existing| {
            if guard == SyntaxGuard::Ordinary {
                *existing = guard;
            }
        })
        .or_insert(guard);
}

pub(super) fn insert_primary_alias(
    aliases: &mut BTreeMap<String, String>,
    guards: &mut BTreeMap<String, SyntaxGuard>,
    unresolved: &mut BTreeSet<String>,
    alias: String,
    target: String,
    conditional: bool,
    guard: SyntaxGuard,
) {
    match guards.get(&alias).copied() {
        None => {
            aliases.insert(alias.clone(), target);
            guards.insert(alias.clone(), guard);
        }
        Some(SyntaxGuard::TestOnly) if guard == SyntaxGuard::Ordinary => {
            aliases.insert(alias.clone(), target);
            guards.insert(alias.clone(), guard);
        }
        Some(SyntaxGuard::Ordinary) if guard == SyntaxGuard::TestOnly => {}
        Some(_) if aliases.get(&alias) != Some(&target) => {
            unresolved.insert(alias.clone());
        }
        Some(_) => {}
    }
    if conditional && guard.is_conditional() {
        unresolved.insert(alias);
    }
}

pub(super) fn visible_root(path: &str) -> &str {
    let root = path.split("::").next().unwrap_or(path);
    root.strip_prefix("r#").unwrap_or(root)
}

pub(super) fn join_path(mut prefix: String, remainder: &[String]) -> String {
    if !remainder.is_empty() {
        prefix.push_str("::");
        prefix.push_str(&remainder.join("::"));
    }
    prefix
}
