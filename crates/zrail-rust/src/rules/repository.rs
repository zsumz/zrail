//! Repository topology and exact directory ownership.

use std::{collections::BTreeSet, fs, path::Path};

use zrail_core::{
    AnalysisQuality, Finding, FindingSink, OwnerKind, PolicyMode, SymlinkMode, path::glob_matches,
};

use crate::inventory::RepositoryEntryKind;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    check_roots(context, findings);
    check_exclusions(context, findings);
    check_nested_git(context, findings);
    check_submodules(context, findings);
    check_symlinks(context, findings);
    check_directory_owners(context, findings);
}

fn check_roots(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for root in &context.contract.repository.roots {
        if !context.inventory.root.join(root).is_dir() {
            findings.push(
                Finding::error(
                    "REP-001",
                    "repository.roots",
                    "repository",
                    format!("declared repository root {root:?} does not exist"),
                )
                .at(root, None),
            );
        }
    }
}

fn check_exclusions(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for pattern in &context.contract.repository.exclude {
        if !context
            .inventory
            .entries
            .iter()
            .any(|entry| glob_matches(pattern, &entry.relative))
        {
            findings.push(Finding::error(
                "REP-006",
                "repository.exclude",
                "repository",
                format!("repository exclusion {pattern:?} matches no path"),
            ));
        }
    }
}

fn check_nested_git(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.repository.nested_git != PolicyMode::Deny {
        return;
    }
    for entry in &context.inventory.entries {
        if entry.relative != ".git" && entry.relative.ends_with("/.git") {
            findings.push(
                Finding::error(
                    "REP-002",
                    "repository.nested-git",
                    "repository",
                    "nested Git metadata creates a second repository boundary",
                )
                .at(&entry.relative, None)
                .with_help("remove the nested repository or declare a separate checkout"),
            );
        }
    }
}

fn check_submodules(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.repository.submodules != PolicyMode::Deny {
        return;
    }
    if context
        .inventory
        .entries
        .iter()
        .any(|entry| entry.relative == ".gitmodules")
    {
        findings.push(
            Finding::error(
                "REP-003",
                "repository.submodules",
                "repository",
                "Git submodules are denied by the architecture contract",
            )
            .at(".gitmodules", None),
        );
    }
}

fn check_symlinks(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for entry in context
        .inventory
        .entries
        .iter()
        .filter(|entry| entry.kind == RepositoryEntryKind::Symlink)
    {
        let target_inside = fs::canonicalize(&entry.absolute)
            .is_ok_and(|target| target.starts_with(&context.inventory.root));
        let denied = context.contract.repository.symlinks == SymlinkMode::Deny || !target_inside;
        if denied {
            findings.push(
                Finding::error(
                    "REP-004",
                    "repository.symlinks",
                    "repository",
                    "symlink escapes or is denied by the repository boundary",
                )
                .at(&entry.relative, None)
                .with_analysis(AnalysisQuality::Exact),
            );
        } else if architecture_input(context, &entry.relative) {
            findings.push(
                Finding::error(
                    "REP-005",
                    "repository.symlinks",
                    "repository",
                    "symlink aliases architecture input that cannot be indexed exactly",
                )
                .at(&entry.relative, None)
                .with_analysis(AnalysisQuality::Unresolved)
                .with_help(
                    "use a regular repository file; asset-only internal symlinks remain allowed",
                ),
            );
        }
    }
}

fn architecture_input(context: &RuleContext<'_>, path: &str) -> bool {
    let architecture_extension = Path::new(path).extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("rs") || extension.eq_ignore_ascii_case("toml")
    });
    architecture_extension
        || path.ends_with("Cargo.lock")
        || context
            .contract
            .repository
            .roots
            .iter()
            .any(|root| root == "." || path == root || path.starts_with(&format!("{root}/")))
}

fn check_directory_owners(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let directories = context
        .inventory
        .entries
        .iter()
        .filter(|entry| entry.kind == RepositoryEntryKind::Directory)
        .map(|entry| entry.relative.as_str())
        .collect::<BTreeSet<_>>();
    for owner in context
        .contract
        .owners
        .iter()
        .filter(|owner| owner.kind == OwnerKind::Directory)
    {
        let allowed = owner
            .allow
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for path in directories
            .iter()
            .filter(|path| glob_matches(&owner.selector, path))
        {
            if !allowed.contains(path) {
                findings.push(
                    Finding::error(
                        "OWN-001",
                        &owner.name,
                        "ownership",
                        format!("directory {path:?} is outside its declared owner"),
                    )
                    .at(*path, None)
                    .because(&owner.reason)
                    .with_help(format!("move it under one of: {}", owner.allow.join(", "))),
                );
            }
        }
        for path in allowed.difference(&directories) {
            findings.push(
                Finding::error(
                    "OWN-002",
                    &owner.name,
                    "ownership",
                    format!("owner policy names missing directory {path:?}"),
                )
                .at(*path, None)
                .because(&owner.reason),
            );
        }
    }
}
