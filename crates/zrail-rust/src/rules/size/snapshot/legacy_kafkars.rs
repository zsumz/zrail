//! Byte-exact frozen Kafkars selectors and debt predicates, used only by trusted tests.

use super::Budget;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub(super) enum FileClass {
    Facade,
    Implementation,
    Test,
    Auxiliary,
}

pub(crate) fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn is_facade(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some("lib.rs" | "mod.rs")
    )
}

pub(crate) fn is_integration_test(package_roots: &[PathBuf], path: &Path) -> bool {
    package_roots.iter().any(|package_root| {
        path.strip_prefix(package_root)
            .ok()
            .and_then(|relative| relative.components().next())
            .is_some_and(|component| component.as_os_str() == "tests")
    })
}

pub(crate) fn classify_with_package_roots(
    root: &Path,
    package_roots: &[PathBuf],
    path: &Path,
) -> FileClass {
    let relative = display_path(root, path);
    if is_integration_test(package_roots, path) || relative.ends_with("_test.rs") {
        FileClass::Test
    } else if relative.contains("/examples/")
        || relative.ends_with("/src/main.rs")
        || relative.contains("/src/bin/")
    {
        FileClass::Auxiliary
    } else if is_facade(path) {
        FileClass::Facade
    } else {
        FileClass::Implementation
    }
}

fn check_baseline(
    relative: &str,
    lines: usize,
    budget: Budget,
    baselines: &BTreeMap<&str, &super::BudgetBaseline>,
    violations: &mut Vec<String>,
) {
    if lines > budget.target {
        match baselines.get(relative) {
            Some(entry) if entry.reason.trim().is_empty() => {
                violations.push(format!("{relative} has an unexplained baseline"));
            }
            Some(entry) if lines != entry.lines => {
                let direction = if lines > entry.lines {
                    "grew beyond"
                } else {
                    "shrunk below"
                };
                violations.push(format!(
                    "{relative} {direction} its exact {}-line baseline to {lines} lines",
                    entry.lines
                ));
            }
            Some(_) => {}
            None => violations.push(format!(
                "{relative} is {lines} lines, above its {}-line design target{}",
                budget.target,
                if lines > budget.soft {
                    " and soft limit"
                } else {
                    ""
                }
            )),
        }
    } else if baselines.contains_key(relative) {
        violations.push(format!("{relative} has a stale baseline"));
    }
}

fn check_allow(
    relative: &str,
    lines: usize,
    budget: Budget,
    allows: &BTreeMap<&str, &super::BudgetAllow>,
    violations: &mut Vec<String>,
) {
    if lines > budget.hard {
        match allows.get(relative) {
            Some(entry)
                if !entry.reason.trim().is_empty()
                    && !entry.owner.trim().is_empty()
                    && !entry.issue.trim().is_empty() => {}
            _ => violations.push(format!(
                "{relative} exceeds its {}-line hard ceiling without a reviewed allow",
                budget.hard
            )),
        }
    } else if allows.contains_key(relative) {
        violations.push(format!("{relative} has a stale hard-ceiling allow"));
    }
}

pub(super) fn violations(
    path: &str,
    lines: usize,
    budget: Budget,
    baseline: Option<&super::BudgetBaseline>,
    allowance: Option<&super::BudgetAllow>,
) -> Vec<String> {
    let mut findings = Vec::new();
    let baselines = baseline.map(|entry| (path, entry)).into_iter().collect();
    let allows = allowance.map(|entry| (path, entry)).into_iter().collect();
    check_baseline(path, lines, budget, &baselines, &mut findings);
    check_allow(path, lines, budget, &allows, &mut findings);
    findings
}

fn collect_roots(
    base: &Path,
    roots: &[String],
    excluded: &[PathBuf],
    scope: WalkScope,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in roots {
        let path = base.join(root);
        assert!(
            path.is_dir(),
            "configured Rust root {} is missing",
            path.display()
        );
        collect(&path, excluded, &mut files);
    }
    files.sort();
    files.dedup();
    if scope == WalkScope::Workspace {
        assert!(
            files.len() >= WORKSPACE_RUST_FILE_FLOOR,
            "workspace traversal found only {} Rust files; expected at least {WORKSPACE_RUST_FILE_FLOOR}",
            files.len()
        );
    }
    files
}

fn collect(root: &Path, excluded: &[PathBuf], files: &mut Vec<PathBuf>) {
    if excluded.iter().any(|path| root == path) {
        return;
    }
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("read directory {}: {error}", root.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("read entry under {}: {error}", root.display()))
            .path();
        if path.is_dir() {
            collect(&path, excluded, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum WalkScope {
    Workspace,
}
const WORKSPACE_RUST_FILE_FLOOR: usize = 20;

pub(super) fn paths(root: &Path, roots: &[String], excluded: &[PathBuf]) -> Vec<PathBuf> {
    collect_roots(root, roots, excluded, WalkScope::Workspace)
}
