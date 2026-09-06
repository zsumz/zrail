//! File-policy selections reuse physical traversal without inheriting Rust exclusions.

use std::{collections::BTreeMap, path::Path};

use zrail_core::{RepositoryFileRule, glob_can_match_descendant, glob_matches};

use crate::inventory::{RepositoryEntry, RepositoryEntryKind, scan_file_entries, skip_directory};

use super::{boundary, model::GovernedRepositoryFileEntry};

const MAX_SELECTION_QUERIES: usize = 8_000_000;
const MAX_SELECTED_ENTRIES: usize = 250_000;

pub(super) struct Selection {
    entries: BTreeMap<String, RepositoryEntry>,
    queries: usize,
    selected: usize,
}

impl Selection {
    pub(super) fn new(root: &Path, rules: &[RepositoryFileRule]) -> Result<Self, String> {
        let entries = if rules
            .iter()
            .any(|rule| rule.include.iter().any(|pattern| is_glob(pattern)))
        {
            scan_file_entries(root).map_err(|error| error.to_string())?
        } else {
            Vec::new()
        };
        Ok(Self {
            entries: entries
                .into_iter()
                .map(|entry| (entry.relative.clone(), entry))
                .collect(),
            queries: 0,
            selected: 0,
        })
    }

    pub(super) fn select(
        &mut self,
        root: &Path,
        rule: &RepositoryFileRule,
    ) -> Result<Vec<GovernedRepositoryFileEntry>, String> {
        let mut candidates = BTreeMap::new();
        for pattern in &rule.include {
            if !is_glob(pattern) {
                if let Some(entry) = boundary::probe(root, pattern)? {
                    candidates.insert(pattern.clone(), entry);
                }
                continue;
            }
            for entry in self.entries.values() {
                charge(&mut self.queries)?;
                if glob_matches(pattern, &entry.relative) {
                    candidates.insert(entry.relative.clone(), entry.clone());
                }
                let unvisited = (entry.kind == RepositoryEntryKind::Directory
                    && skip_directory(&entry.relative))
                    || entry.kind == RepositoryEntryKind::Symlink;
                if unvisited {
                    charge(&mut self.queries)?;
                    if glob_can_match_descendant(pattern, &entry.relative)?
                        && !subtree_excluded(&rule.exclude, &entry.relative, &mut self.queries)?
                    {
                        // A contained file link has no descendants. Every other unread
                        // target remains uncertain, including broken and escaping links.
                        let file_link = entry.kind == RepositoryEntryKind::Symlink
                            && boundary::contained(root, &entry.absolute)
                                .is_ok_and(|target| target.is_file());
                        if !file_link {
                            return Err(format!(
                                "selector {pattern:?} may match unread descendants of {:?}; use exact paths or an explicit subtree exclusion",
                                entry.relative
                            ));
                        }
                    }
                }
            }
        }
        let mut selected = Vec::new();
        for entry in candidates.values() {
            if excluded(&rule.exclude, &entry.relative, &mut self.queries)? {
                continue;
            }
            if let Some(observation) = boundary::observe(root, entry, rule.entry)? {
                self.selected += 1;
                if self.selected > MAX_SELECTED_ENTRIES {
                    return Err(format!(
                        "file policies exceed the {MAX_SELECTED_ENTRIES}-entry observation limit"
                    ));
                }
                selected.push(observation);
            }
        }
        Ok(selected)
    }
}

fn is_glob(pattern: &str) -> bool {
    pattern.contains(['*', '?'])
}

fn charge(queries: &mut usize) -> Result<(), String> {
    *queries += 1;
    if *queries > MAX_SELECTION_QUERIES {
        Err(format!(
            "file policies exceed the {MAX_SELECTION_QUERIES}-query selection limit"
        ))
    } else {
        Ok(())
    }
}

fn excluded(patterns: &[String], path: &str, queries: &mut usize) -> Result<bool, String> {
    for pattern in patterns {
        charge(queries)?;
        if glob_matches(pattern, path) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn subtree_excluded(patterns: &[String], path: &str, queries: &mut usize) -> Result<bool, String> {
    for pattern in patterns {
        charge(queries)?;
        if (pattern == "**" || pattern.ends_with("/**")) && glob_matches(pattern, path) {
            return Ok(true);
        }
    }
    Ok(false)
}
