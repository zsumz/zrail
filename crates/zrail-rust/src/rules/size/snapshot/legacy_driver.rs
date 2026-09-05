//! Byte-exact frozen Kafka-driver selectors, used only by trusted tests.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
pub(super) struct Budgets {
    pub(super) facade: usize,
    pub(super) test: usize,
    pub(super) production: usize,
}

pub(crate) fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn is_facade(root: &Path, path: &Path) -> bool {
    let name = path.file_name().and_then(|name| name.to_str());
    matches!(name, Some("lib.rs" | "mod.rs")) || display_path(root, path) == "tests/guardrails.rs"
}

pub(crate) fn is_test(root: &Path, path: &Path) -> bool {
    let relative = display_path(root, path);
    relative.starts_with("tests/") || relative.ends_with("_test.rs")
}

fn limit_for(root: &Path, path: &Path, budgets: Budgets) -> usize {
    if is_facade(root, path) {
        budgets.facade
    } else if is_test(root, path) {
        budgets.test
    } else {
        budgets.production
    }
}

pub(super) fn limit(root: &Path, path: &Path, budgets: Budgets) -> usize {
    limit_for(root, path, budgets)
}

pub(super) fn violations(root: &Path, path: &Path, budgets: Budgets, lines: usize) -> Vec<String> {
    let root = root.to_path_buf();
    let source = "\n".repeat(lines);
    let read = |_: &std::path::PathBuf| source.as_str();
    std::iter::once(path.to_path_buf())
        .filter_map(|path| {
            let lines = read(&path).lines().count();
            let limit = limit_for(&root, &path, budgets);
            (lines > limit).then(|| {
                format!(
                    "{}:{lines} exceeds its {limit}-line ceiling",
                    display_path(&root, &path)
                )
            })
        })
        .collect()
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read directory {}: {error}", directory.display()))
        .map(|entry| entry.unwrap_or_else(|error| panic!("read directory entry: {error}")))
        .collect::<Vec<_>>();
    entries.sort_by_key(fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("inspect {}: {error}", path.display()));
        if file_type.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

pub(super) fn paths(root: &Path, roots: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for rust_root in roots {
        collect_rust_files(&root.join(rust_root), &mut files);
    }
    files.sort();
    files
}
