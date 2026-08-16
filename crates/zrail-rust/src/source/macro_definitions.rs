//! Bounded package-level shadowing evidence for lexical macro definitions.

use std::collections::{BTreeMap, BTreeSet};

use crate::cargo::Package;

use super::SourceIndex;

pub(super) const MAX_MACRO_DEFINITIONS_PER_PACKAGE: usize = 256;

#[derive(Default)]
pub(super) struct MacroDefinitionSet {
    pub(super) names: BTreeSet<String>,
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
        let mut file_names = BTreeSet::new();
        for definition in &file.macro_definitions {
            file_names.insert(definition.name.clone());
            if file_names.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
                file_names.clear();
                break;
            }
        }
        for package in packages {
            merge_file_definitions(
                definitions.entry(package.clone()).or_default(),
                &file_names,
                !file.macro_definitions.is_empty(),
            );
        }
    }
    definitions
}

fn merge_file_definitions(
    entry: &mut MacroDefinitionSet,
    names: &BTreeSet<String>,
    had_definitions: bool,
) {
    if names.is_empty() && had_definitions {
        entry.overflowed = true;
        entry.names.clear();
    } else if !entry.overflowed {
        entry.names.extend(names.iter().cloned());
        if entry.names.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
            entry.overflowed = true;
            entry.names.clear();
        }
    }
}

pub(super) fn local_macro_names<'a>(
    packages: &[&Package],
    definitions: &'a BTreeMap<String, MacroDefinitionSet>,
) -> Option<BTreeSet<&'a str>> {
    let mut names = BTreeSet::new();
    for package in packages {
        let Some(definitions) = definitions.get(package.name.as_str()) else {
            continue;
        };
        if definitions.overflowed {
            return None;
        }
        names.extend(definitions.names.iter().map(String::as_str));
        if names.len() > MAX_MACRO_DEFINITIONS_PER_PACKAGE {
            return None;
        }
    }
    Some(names)
}
