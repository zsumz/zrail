//! Rust source classification for budgets and source-shape rules.

use std::{ffi::OsStr, path::Path};

use zrail_core::GeneratedSourceContract;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum FileClass {
    Facade,
    Implementation,
    Test,
    Auxiliary,
    EntryPoint,
    Generated,
}

pub(crate) fn classify_path(relative: &str, generated: &[GeneratedSourceContract]) -> FileClass {
    let path = Path::new(relative);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if generated
        .iter()
        .any(|source| under_root(relative, &source.root))
    {
        FileClass::Generated
    } else if has_component(path, "tests") || name.ends_with("_test.rs") {
        FileClass::Test
    } else if name == "main.rs" {
        FileClass::EntryPoint
    } else if matches!(name, "lib.rs" | "mod.rs") {
        FileClass::Facade
    } else if has_component(path, "examples") || contains_components(path, "src", "bin") {
        FileClass::Auxiliary
    } else {
        FileClass::Implementation
    }
}

pub(crate) fn under_root(path: &str, root: &str) -> bool {
    root == "." || path == root || path.starts_with(&format!("{root}/"))
}

pub(super) fn is_indexed_source(relative: &str, generated: &[GeneratedSourceContract]) -> bool {
    let extension = Path::new(relative)
        .extension()
        .and_then(|extension| extension.to_str());
    extension.is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        || extension.is_some_and(|extension| extension == "rsi")
            && generated
                .iter()
                .any(|source| under_root(relative, &source.root))
}

fn has_component(path: &Path, expected: &str) -> bool {
    let expected = OsStr::new(expected);
    path.components()
        .any(|component| component.as_os_str() == expected)
}

fn contains_components(path: &Path, first: &str, second: &str) -> bool {
    let first = OsStr::new(first);
    let second = OsStr::new(second);
    let mut previous_was_first = false;
    for component in path.components() {
        let current = component.as_os_str();
        if previous_was_first && current == second {
            return true;
        }
        previous_was_first = current == first;
    }
    false
}

#[cfg(test)]
#[path = "classify_test.rs"]
mod classify_test;
