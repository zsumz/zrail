//! Effective source policy is shared by enforcement and agent explanations.

use zrail_core::{Budget, FacadeMode, FileRole, RustSourceContract};

use crate::{
    inventory::{FileClass, under_root},
    source::Reachability,
};

pub(crate) fn budget_for(
    path: &str,
    class: FileClass,
    reachability: Reachability,
    rust: &RustSourceContract,
) -> Option<Budget> {
    let size = rust.size.as_ref();
    if class != FileClass::Generated && reachability.is_test_only() {
        return size.map(|size| size.test);
    }
    let class = effective_file_role(path, class, rust).effective;
    match class {
        FileClass::Facade => size.map(|size| size.facade),
        FileClass::Implementation | FileClass::Test => size.map(|size| size.implementation),
        FileClass::Auxiliary | FileClass::EntryPoint => size.map(|size| size.auxiliary),
        FileClass::Generated => rust
            .generated
            .iter()
            .find(|generated| under_root(path, &generated.root))
            .map(|generated| Budget {
                target: generated.target,
                hard: generated.hard,
            }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EffectiveFileRole<'a> {
    pub(crate) inferred: FileClass,
    pub(crate) effective: FileClass,
    pub(crate) reason: Option<&'a str>,
}

pub(crate) fn effective_file_role<'a>(
    path: &str,
    inferred: FileClass,
    rust: &'a RustSourceContract,
) -> EffectiveFileRole<'a> {
    if !matches!(
        inferred,
        FileClass::Facade | FileClass::Implementation | FileClass::EntryPoint
    ) {
        return EffectiveFileRole {
            inferred,
            effective: inferred,
            reason: None,
        };
    }
    let declared = rust.file_roles.iter().find(|role| {
        role.path == path
            && role.role != FileRole::TestFacade
            && (inferred != FileClass::EntryPoint || role.role == FileRole::Implementation)
    });
    let effective = declared.map_or(inferred, |declared| match declared.role {
        FileRole::Facade => FileClass::Facade,
        FileRole::Implementation => FileClass::Implementation,
        FileRole::TestFacade => inferred,
    });
    EffectiveFileRole {
        inferred,
        effective,
        reason: declared.map(|declared| declared.reason.as_str()),
    }
}

/// Structural policy never changes compilation reachability or test placement.
pub(crate) fn facade_mode_for(
    path: &str,
    inferred: FileClass,
    rust: &RustSourceContract,
) -> Option<FacadeMode> {
    let declared = rust.file_roles.iter().find(|role| role.path == path);
    if let Some(declared) = declared {
        if declared.role == FileRole::TestFacade {
            return declared.mode;
        }
        if declared.role == FileRole::Facade
            && matches!(inferred, FileClass::Facade | FileClass::Implementation)
        {
            return Some(declared.mode.unwrap_or(rust.facades));
        }
    }
    match effective_file_role(path, inferred, rust).effective {
        FileClass::Facade => Some(rust.facades),
        FileClass::EntryPoint => Some(rust.entrypoints),
        _ => None,
    }
}

pub(crate) const fn facade_mode_name(mode: FacadeMode) -> &'static str {
    match mode {
        FacadeMode::Allow => "allow",
        FacadeMode::Declarative => "declarative",
        FacadeMode::WiringOnly => "wiring-only",
        FacadeMode::WiringReexports => "wiring-reexports",
    }
}

pub(crate) const fn role_name(role: FileClass) -> &'static str {
    match role {
        FileClass::Facade => "facade",
        FileClass::Implementation => "implementation",
        FileClass::Test => "test",
        FileClass::Auxiliary => "auxiliary",
        FileClass::EntryPoint => "entrypoint",
        FileClass::Generated => "generated",
    }
}

#[cfg(test)]
#[path = "source_policy_test.rs"]
mod source_policy_test;
