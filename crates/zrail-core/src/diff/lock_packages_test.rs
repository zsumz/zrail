//! Protected review preserves exact inventory identities, quantities, and zero bans.

use crate::{ChangeKind, Contract, LockPackageRule, compare_architecture};

fn contract(assertion: &str) -> Contract {
    let mut contract = super::super::compare_fixture_test::contract_with_hard_limit(300);
    contract.dependencies.lock_packages.push(toml::from_str::<LockPackageRule>(&format!(
        "name = 'wire'\npackage = 'wire'\nreason = 'Reviewed protocol provenance.'\nassertion = {{ {assertion} }}"
    )).expect("lock package contract"));
    contract
}

fn exact(versions: &[&str]) -> String {
    format!(
        "kind = 'exact', identities = [{}]",
        versions
            .iter()
            .map(|version| { format!("{{ version = '{version}', source = 'path+crates/wire' }}") })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn kinds(before: &Contract, after: &Contract) -> Vec<ChangeKind> {
    let report = compare_architecture(before, None, after, None);
    let kinds = report
        .changes
        .iter()
        .filter(|change| {
            assert_eq!(change.rail, "dependencies.lock-package");
            assert_eq!(change.subject, "dependency:lock-package:wire");
            true
        })
        .map(|change| change.kind)
        .collect::<Vec<_>>();
    if kinds.contains(&ChangeKind::Grant) || kinds.contains(&ChangeKind::Unknown) {
        assert!(report.denies_grants());
    }
    kinds
}

#[test]
fn identity_removal_substitution_and_exact_count_changes_are_protected() {
    for (before, after) in [
        (exact(&["1.0.0", "2.0.0"]), exact(&["1.0.0"])),
        (exact(&["1.0.0"]), exact(&["2.0.0"])),
        (
            "kind = 'count', count = 2".into(),
            "kind = 'count', count = 1".into(),
        ),
        (
            "kind = 'count', count = 0".into(),
            "kind = 'count', count = 1".into(),
        ),
    ] {
        assert_eq!(
            kinds(&contract(&before), &contract(&after)),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
    }
    let before = contract(&exact(&["1.0.0"]));
    for changed in [
        exact(&["1.0.0"]).replace("path+crates/wire", "path+other"),
        format!(
            "kind = 'exact', identities = [{{ version = '1.0.0', source = 'path+crates/wire', checksum = '{}' }}]",
            "a".repeat(64)
        ),
    ] {
        assert_eq!(
            kinds(&before, &contract(&changed)),
            [ChangeKind::Grant, ChangeKind::Revoke]
        );
    }
}

#[test]
fn replacing_exact_identities_with_their_quantity_is_a_grant_except_for_zero() {
    let before = contract(&exact(&["1.0.0"]));
    let after = contract("kind = 'count', count = 1");
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Revoke]);
    assert!(
        kinds(
            &contract(&exact(&[])),
            &contract("kind = 'count', count = 0")
        )
        .is_empty()
    );
    assert!(
        kinds(
            &contract(&exact(&["1.0.0", "2.0.0"])),
            &contract(&exact(&["2.0.0", "1.0.0"]))
        )
        .is_empty()
    );
}

#[test]
fn removing_retargeting_and_rejustifying_a_guard_require_review() {
    let before = contract("kind = 'count', count = 0");
    let mut after = before.clone();
    after.dependencies.lock_packages.clear();
    assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
    after = before.clone();
    after.dependencies.lock_packages[0].package = "elsewhere".into();
    assert_eq!(
        kinds(&before, &after),
        [ChangeKind::Grant, ChangeKind::Revoke]
    );
    after = before.clone();
    after.dependencies.lock_packages[0].reason = "Different reviewed intent.".into();
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
}
