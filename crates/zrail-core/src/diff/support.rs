//! Small deterministic helpers shared by semantic comparisons.

use std::collections::BTreeSet;

use crate::{
    CycleMode, DependencyMode, ExternalDependencyMode, FacadeMode, GlobImportMode,
    LintSuppressionMode, MacroExpansionMode, ModuleDocsMode, PolicyMode, SymlinkMode, TestMode,
};

use super::{ArchitectureChange, ChangeKind};

pub(super) fn compare_number(
    rail: &str,
    subject: &str,
    before: usize,
    after: usize,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before == after {
        return;
    }
    let kind = if after > before {
        ChangeKind::Grant
    } else {
        ChangeKind::Revoke
    };
    changes.push(
        ArchitectureChange::new(kind, rail, subject, "source-size permission changed")
            .values(before.to_string(), after.to_string()),
    );
}

pub(super) fn compare_ordered_mode(
    rail: &str,
    subject: &str,
    before: u8,
    after: u8,
    changes: &mut Vec<ArchitectureChange>,
) {
    if before == after {
        return;
    }
    let kind = if after < before {
        ChangeKind::Grant
    } else {
        ChangeKind::Revoke
    };
    changes.push(ArchitectureChange::new(
        kind,
        rail,
        subject,
        "enforcement mode changed",
    ));
}

pub(super) fn compare_named_set(
    rail: &str,
    owner: &str,
    before: &[String],
    after: &[String],
    added: ChangeKind,
    removed: ChangeKind,
    verb: &str,
    changes: &mut Vec<ArchitectureChange>,
) {
    let left = before.iter().cloned().collect::<BTreeSet<_>>();
    let right = after.iter().cloned().collect::<BTreeSet<_>>();
    for value in right.difference(&left) {
        changes.push(ArchitectureChange::new(
            added,
            rail,
            format!("{owner}:{value}"),
            format!("contract now {verb}"),
        ));
    }
    for value in left.difference(&right) {
        changes.push(ArchitectureChange::new(
            removed,
            rail,
            format!("{owner}:{value}"),
            format!("contract no longer {verb}"),
        ));
    }
}

pub(super) fn compare_set_values(
    rail: &str,
    before: &BTreeSet<String>,
    after: &BTreeSet<String>,
    added: ChangeKind,
    removed: ChangeKind,
    verb: &str,
    changes: &mut Vec<ArchitectureChange>,
) {
    for value in after.difference(before) {
        changes.push(ArchitectureChange::new(
            added,
            rail,
            value,
            format!("contract now {verb}"),
        ));
    }
    for value in before.difference(after) {
        changes.push(ArchitectureChange::new(
            removed,
            rail,
            value,
            format!("contract no longer {verb}"),
        ));
    }
}

pub(super) const fn rank_module_docs(mode: ModuleDocsMode) -> u8 {
    match mode {
        ModuleDocsMode::Allow => 0,
        ModuleDocsMode::Required => 1,
    }
}

pub(super) const fn rank_facades(mode: FacadeMode) -> u8 {
    match mode {
        FacadeMode::Allow => 0,
        FacadeMode::Declarative => 1,
    }
}

pub(super) const fn rank_tests(mode: TestMode) -> u8 {
    match mode {
        TestMode::Allow => 0,
        TestMode::Sibling => 1,
    }
}

pub(super) const fn rank_policy(mode: PolicyMode) -> u8 {
    match mode {
        PolicyMode::Allow => 0,
        PolicyMode::Deny => 1,
    }
}

pub(super) const fn rank_lint_suppressions(mode: LintSuppressionMode) -> u8 {
    match mode {
        LintSuppressionMode::Allow => 0,
        LintSuppressionMode::Reasoned => 1,
        LintSuppressionMode::Deny => 2,
    }
}

pub(super) const fn rank_glob_imports(mode: GlobImportMode) -> u8 {
    match mode {
        GlobImportMode::Allow => 0,
        GlobImportMode::FacadeReexportsAndTestSuper => 1,
        GlobImportMode::FacadeReexportsOnly => 2,
        GlobImportMode::Deny => 3,
    }
}

pub(super) const fn rank_macro_expansion(mode: MacroExpansionMode) -> u8 {
    match mode {
        MacroExpansionMode::Allow => 0,
        MacroExpansionMode::DenyUnreviewed => 1,
    }
}

pub(super) const fn rank_symlinks(mode: SymlinkMode) -> u8 {
    match mode {
        SymlinkMode::Inside => 0,
        SymlinkMode::Deny => 1,
    }
}

pub(super) const fn rank_dependencies(mode: DependencyMode) -> u8 {
    match mode {
        DependencyMode::Observed => 0,
        DependencyMode::Locked => 1,
    }
}

pub(super) const fn rank_cycles(mode: CycleMode) -> u8 {
    match mode {
        CycleMode::Allow => 0,
        CycleMode::Deny => 1,
    }
}

pub(super) const fn rank_external_dependencies(mode: ExternalDependencyMode) -> u8 {
    match mode {
        ExternalDependencyMode::Allow => 0,
        ExternalDependencyMode::Locked => 1,
        ExternalDependencyMode::None => 2,
    }
}
