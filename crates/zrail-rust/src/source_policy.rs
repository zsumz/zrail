//! Effective source policy is shared by enforcement and agent explanations.

use zrail_core::{Budget, FileRole, RustSourceContract};

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
    if !matches!(inferred, FileClass::Facade | FileClass::Implementation) {
        return EffectiveFileRole {
            inferred,
            effective: inferred,
            reason: None,
        };
    }
    let declared = rust.file_roles.iter().find(|role| role.path == path);
    let effective = declared.map_or(inferred, |declared| match declared.role {
        FileRole::Facade => FileClass::Facade,
        FileRole::Implementation => FileClass::Implementation,
    });
    EffectiveFileRole {
        inferred,
        effective,
        reason: declared.map(|declared| declared.reason.as_str()),
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
