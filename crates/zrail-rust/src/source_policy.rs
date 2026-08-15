//! Effective source policy is shared by enforcement and agent explanations.

use zrail_core::{Budget, RustSourceContract};

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
    if class != FileClass::Generated && reachability == Reachability::TestOnly {
        return size.map(|size| size.test);
    }
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

#[cfg(test)]
#[path = "source_policy_test.rs"]
mod source_policy_test;
