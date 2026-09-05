//! File-role override semantic comparison.

use crate::{ChangeKind, FileRole, FileRoleContract, compare_architecture};

use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn implementation_overrides_grant_and_facade_overrides_revoke() {
    let base = contract_with_hard_limit(300);
    let mut implementation = base.clone();
    implementation.source.rust.file_roles = vec![role("src/lib.rs", FileRole::Implementation)];
    let mut facade = base.clone();
    facade.source.rust.file_roles = vec![role("src/api.rs", FileRole::Facade)];

    let implementation = compare_architecture(&base, None, &implementation, None);
    let facade = compare_architecture(&base, None, &facade, None);

    assert!(
        implementation
            .changes
            .iter()
            .any(|change| { change.kind == ChangeKind::Grant && change.rail == "rust.file-role" })
    );
    assert!(
        facade
            .changes
            .iter()
            .any(|change| { change.kind == ChangeKind::Revoke && change.rail == "rust.file-role" })
    );
}

#[test]
fn every_facade_mode_weakening_is_a_protected_grant() {
    let modes = [
        crate::FacadeMode::Allow,
        crate::FacadeMode::Declarative,
        crate::FacadeMode::WiringOnly,
        crate::FacadeMode::WiringReexports,
    ];
    for (index, left) in modes.iter().enumerate() {
        for right in &modes[..index] {
            let mut before = contract_with_hard_limit(300);
            before.source.rust.facades = *left;
            let mut after = before.clone();
            after.source.rust.facades = *right;
            assert!(compare_architecture(&before, None, &after, None).changes.iter().any(|change|
                change.rail == "rust.facades" && change.kind == ChangeKind::Grant));
            before.source.rust.file_roles = vec![role("src/api.rs", FileRole::Facade)];
            before.source.rust.file_roles[0].mode = Some(*left);
            after = before.clone();
            after.source.rust.file_roles[0].mode = Some(*right);
            assert!(compare_architecture(&before, None, &after, None).changes.iter().any(|change|
                change.rail == "rust.facades" && change.kind == ChangeKind::Grant));
        }
    }
}

#[test]
fn removing_or_redirecting_test_facade_structure_requires_review() {
    let mut before = contract_with_hard_limit(300);
    let mut test = role("tests/suite.rs", FileRole::TestFacade);
    test.mode = Some(crate::FacadeMode::WiringOnly);
    before.source.rust.file_roles = vec![test];
    let mut after = before.clone();
    after.source.rust.file_roles.clear();
    let diff = compare_architecture(&before, None, &after, None);
    assert!(
        diff.changes
            .iter()
            .any(|change| change.rail == "rust.file-role" && change.kind == ChangeKind::Grant)
    );
    after = before.clone();
    after.source.rust.file_roles[0].path = "tests/replacement.rs".into();
    let diff = compare_architecture(&before, None, &after, None);
    assert!(
        diff.changes
            .iter()
            .any(|change| change.rail == "rust.file-role" && change.kind == ChangeKind::Grant)
    );
}

#[test]
fn adding_a_test_facade_does_not_inherit_a_production_facade_mode() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.facades = crate::FacadeMode::WiringReexports;
    let mut after = before.clone();
    let mut test = role("tests/suite.rs", FileRole::TestFacade);
    test.mode = Some(crate::FacadeMode::WiringOnly);
    after.source.rust.file_roles.push(test);
    let diff = compare_architecture(&before, None, &after, None);
    assert!(
        diff.changes
            .iter()
            .any(|change| change.kind == ChangeKind::Revoke)
    );
    assert!(
        !diff
            .changes
            .iter()
            .any(|change| change.kind == ChangeKind::Grant)
    );
}

fn role(path: &str, role: FileRole) -> FileRoleContract {
    FileRoleContract {
        path: path.into(),
        role,
        mode: None,
        reason: "reviewed source role".into(),
    }
}
