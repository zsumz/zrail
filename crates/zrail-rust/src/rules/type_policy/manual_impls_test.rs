//! Manual impl classification never drops unresolved governed identities.

use std::{collections::BTreeSet, fs};

use zrail_core::{
    CloneCopyPolicy, PolicyReachability, RustTypeContract, RustTypeKind, TypeProhibition,
};

use super::{ManualImplMatch, classify};
use crate::rules::type_policy::identity::IdentityResolution;
use crate::type_policy_test::{TYPE_POLICY, error_count, report, repository, reset, write};

#[test]
fn unresolved_target_with_clone_trait_is_possible_duplication() {
    let target = IdentityResolution::unresolved();
    let trait_identity = exact("std::clone::Clone");

    assert_eq!(
        classify(&policy(), &target, &trait_identity, "Clone"),
        ManualImplMatch::Possible(Some(zrail_core::DuplicationTrait::Clone))
    );
}

#[test]
fn exact_other_target_is_irrelevant_even_when_trait_is_unresolved() {
    let target = exact("other::Ticket");
    let trait_identity = IdentityResolution::unresolved();

    assert_eq!(
        classify(&policy(), &target, &trait_identity, "C"),
        ManualImplMatch::Irrelevant
    );
}

#[test]
fn unresolved_impl_in_another_package_does_not_claim_the_governed_type() {
    let root = repository(TYPE_POLICY);
    write(
        &root,
        "Cargo.toml",
        "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[workspace]\nmembers = [\"other\"]\nresolver = \"3\"\n",
    );
    fs::create_dir_all(root.join("other/src")).expect("create other package");
    write(
        &root,
        "other/Cargo.toml",
        "[package]\nname = \"other\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write(&root, "src/lib.rs", "//! facade\nstruct Ticket;\n");
    write(
        &root,
        "other/src/lib.rs",
        r"//! unrelated package
use self::U as T;
use self::T as U;
impl Clone for T {
    fn clone(&self) -> Self { unreachable!() }
}
",
    );

    let report = report(&root);

    assert_eq!(error_count(&report, "RUST-TYPE-004"), 0, "{report}");
    reset(&root);
}

fn exact(value: &str) -> IdentityResolution {
    IdentityResolution {
        exact: BTreeSet::from([value.into()]),
        unresolved: false,
    }
}

fn policy() -> RustTypeContract {
    RustTypeContract {
        name: "ticket".into(),
        identity: "crate::Ticket".into(),
        path: "src/lib.rs".into(),
        kind: RustTypeKind::Type,
        reachability: PolicyReachability::Production,
        deny: vec![TypeProhibition::ImplClone],
        clone_copy: CloneCopyPolicy::Allow,
        visibility: None,
        leaf_module: None,
        fields: None,
        reason: "Ticket authority is transferred.".into(),
    }
}
