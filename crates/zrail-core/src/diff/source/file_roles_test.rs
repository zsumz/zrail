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

fn role(path: &str, role: FileRole) -> FileRoleContract {
    FileRoleContract {
        path: path.into(),
        role,
        reason: "reviewed source role".into(),
    }
}
