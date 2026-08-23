//! Bounded package-level shadowing evidence for lexical macro definitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::Package;

use super::{SourceIndex, SyntaxGuard};

pub(super) const MAX_MACRO_DEFINITIONS_PER_PACKAGE: usize = 256;

#[derive(Default)]
pub(super) struct MacroDefinitionSet {
    pub(super) ordinary: BTreeSet<String>,
    pub(super) test_only: BTreeSet<String>,
    pub(super) overflowed: bool,
}

pub(super) fn package_macro_definitions(
    index: &SourceIndex,
    contexts: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, MacroDefinitionSet> {
    let mut definitions = BTreeMap::<String, MacroDefinitionSet>::new();
    for file in &index.files {
        let Some(packages) = contexts.get(&file.relative) else {
            continue;
        };
        let mut ordinary = BTreeSet::new();
        let mut test_only = BTreeSet::new();
        for definition in &file.macro_definitions {
            if definition.guard == SyntaxGuard::TestOnly || !file.reachability.is_production() {
                test_only.insert(definition.name.clone());
            } else {
                ordinary.insert(definition.name.clone());
            }
            if ordinary.len() + test_only.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
                ordinary.clear();
                test_only.clear();
                break;
            }
        }
        for package in packages {
            merge_file_definitions(
                definitions.entry(package.clone()).or_default(),
                &ordinary,
                &test_only,
                !file.macro_definitions.is_empty(),
            );
        }
    }
    definitions
}

fn merge_file_definitions(
    entry: &mut MacroDefinitionSet,
    ordinary: &BTreeSet<String>,
    test_only: &BTreeSet<String>,
    had_definitions: bool,
) {
    if ordinary.is_empty() && test_only.is_empty() && had_definitions {
        entry.overflowed = true;
        entry.ordinary.clear();
        entry.test_only.clear();
    } else if !entry.overflowed {
        entry.ordinary.extend(ordinary.iter().cloned());
        entry.test_only.extend(test_only.iter().cloned());
        if entry.ordinary.len() + entry.test_only.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
            entry.overflowed = true;
            entry.ordinary.clear();
            entry.test_only.clear();
        }
    }
}

pub(super) fn local_macro_names<'a>(
    packages: &[&Package],
    definitions: &'a BTreeMap<String, MacroDefinitionSet>,
    context: SyntaxGuard,
) -> Option<BTreeSet<&'a str>> {
    let mut names = BTreeSet::new();
    for package in packages {
        let Some(definitions) = definitions.get(package.name.as_str()) else {
            continue;
        };
        if definitions.overflowed {
            return None;
        }
        names.extend(definitions.ordinary.iter().map(String::as_str));
        if context == SyntaxGuard::TestOnly {
            names.extend(definitions.test_only.iter().map(String::as_str));
        }
        if names.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
            return None;
        }
    }
    Some(names)
}
