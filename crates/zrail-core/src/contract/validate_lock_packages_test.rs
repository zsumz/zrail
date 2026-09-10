//! Strict parsing and bounded lock identities retain old contract defaults.

use crate::{LockPackageAssertion, LockPackageRule};

fn rule(assertion: &str) -> LockPackageRule {
    toml::from_str(&format!(
        "name = 'wire'\npackage = 'wire'\nreason = 'Reviewed protocol provenance.'\nassertion = {{ {assertion} }}"
    )).expect("lock package rule")
}

fn validate(rules: Vec<LockPackageRule>) -> Result<(), crate::ContractError> {
    let mut contract = super::super::validate_fixture_test::minimal_contract();
    contract.dependencies.lock_packages = rules;
    super::super::validate::validate_contract(&contract)
}

#[test]
fn old_dependency_contracts_omit_the_optional_inventory_family() {
    let old = "mode = 'observed'\nunassigned_packages = 'allow'\ncycles = 'allow'";
    let contract: crate::DependenciesContract = toml::from_str(old).expect("rc8 dependency policy");
    assert!(contract.lock_packages.is_empty());
    assert!(
        !toml::to_string(&contract)
            .expect("contract serialization")
            .contains("lock_package")
    );
    for assertion in [
        "kind = 'count', count = 0",
        "kind = 'exact', identities = []",
    ] {
        validate(vec![rule(assertion)]).expect("absent prohibited package remains valid policy");
    }
}

#[test]
fn lock_inventory_schema_rejects_unknown_fields_and_ill_typed_authority() {
    for assertion in [
        "kind = 'count', count = -1",
        "kind = 'count', count = '1'",
        "kind = 'count', count = 1, identities = []",
        "kind = 'exact', identities = [], count = 0",
        "kind = 'reachable', count = 0",
        "kind = 'exact', identities = [{version = '1.0.0', source = 'path+.', optional = true}]",
        "kind = 'exact', identities = [{version = '1.0.0', source = 'path+.', checksum = false}]",
    ] {
        assert!(toml::from_str::<LockPackageRule>(&format!(
            "name = 'wire'\npackage = 'wire'\nreason = 'Reviewed.'\nassertion = {{ {assertion} }}"
        )).is_err(), "{assertion}");
    }
}

#[test]
fn inventory_validation_rejects_ambiguous_identity_sets_and_excessive_counts() {
    let mut duplicate =
        rule("kind = 'exact', identities = [{version = '1.0.0', source = 'path+.'}]");
    let LockPackageAssertion::Exact { identities } = &mut duplicate.assertion else {
        panic!("exact rule")
    };
    identities.push(identities[0].clone());
    assert!(
        validate(vec![duplicate])
            .expect_err("duplicate identity")
            .to_string()
            .contains("duplicate identities")
    );
    let count = rule("kind = 'count', count = 1");
    assert!(
        validate(vec![count.clone(), count])
            .expect_err("duplicate name")
            .to_string()
            .contains("unique nonempty names")
    );
    assert!(
        validate(vec![rule("kind = 'count', count = 100001")])
            .expect_err("bounded count")
            .to_string()
            .contains("100000-node")
    );
    for (field, value, expected) in [
        ("name", " wire", "unique nonempty names"),
        ("package", "wire*", "package name"),
        ("reason", "", "requires a reason"),
    ] {
        let mut invalid = rule("kind = 'count', count = 0");
        match field {
            "name" => invalid.name = value.into(),
            "package" => invalid.package = value.into(),
            _ => invalid.reason = value.into(),
        }
        assert!(
            validate(vec![invalid])
                .expect_err(field)
                .to_string()
                .contains(expected)
        );
    }
    let invalid =
        rule("kind = 'exact', identities = [{version = ' 1.0.0', source = '', checksum = 'BAD'}]");
    let error = validate(vec![invalid])
        .expect_err("literal identities")
        .to_string();
    assert!(error.contains("literal version/source"));
    assert!(error.contains("lowercase SHA-256"));
}
