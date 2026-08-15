//! Qualification and evidence graph contract validation.

use crate::contract::{
    load::{ContractError, ContractFile},
    merge::MergeState,
    validate::validate_contract,
};

const BASE: &str = r#"
schema = 1
adapters = ["rust"]

[repository]
roots = ["src"]
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "allow"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "sibling"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "deny"

[source.rust.size.facade]
target = 80
hard = 120

[source.rust.size.implementation]
target = 240
hard = 300

[source.rust.size.test]
target = 300
hard = 300

[source.rust.size.auxiliary]
target = 300
hard = 300
"#;

#[test]
fn accepts_a_connected_local_and_ci_graph() {
    let extra = r#"
[[gate]]
name = "check"
kind = "local"
path = "scripts/check"
reason = "Canonical local qualification."

[[gate]]
name = "ci"
kind = "ci"
path = ".github/workflows/ci.yml"
requires = ["check"]
reason = "CI delegates to local qualification."

[[invariant]]
id = "ARCH-01"
title = "Architecture is qualified"
status = "enforced"
document = "docs/architecture.md#arch-01"
evidence = ["rust-test:src/architecture_test.rs::qualifies", "gate:ci"]
"#;

    assert!(contract(extra).is_ok());
}

#[test]
fn rejects_cycles_and_higher_gates_disconnected_from_local_qualification() {
    let extra = r#"
[[gate]]
name = "ci-a"
kind = "ci"
path = ".github/workflows/a.yml"
requires = ["ci-b"]
reason = "First half of a broken cycle."

[[gate]]
name = "ci-b"
kind = "ci"
path = ".github/workflows/b.yml"
requires = ["ci-a"]
reason = "Second half of a broken cycle."

[[invariant]]
id = "ARCH-01"
title = "Architecture is qualified"
status = "enforced"
document = "docs/architecture.md#arch-01"
evidence = ["rust-test:src/architecture_test.rs::qualifies", "gate:ci-a"]
"#;

    let error = contract(extra).expect_err("broken graph must fail");
    assert!(error.to_string().contains("cycle"));
    assert!(error.to_string().contains("not connected to a local gate"));
}

#[test]
fn rejects_stale_gates_and_invariants_without_both_evidence_kinds() {
    let extra = r#"
[[gate]]
name = "check"
kind = "local"
path = "scripts/check"
reason = "Unused qualification."

[[invariant]]
id = "ARCH-01"
title = "Architecture is qualified"
status = "enforced"
document = "docs/architecture.md#arch-01"
evidence = ["rust-test:src/architecture_test.rs::qualifies"]
"#;

    let error = contract(extra).expect_err("disconnected graph must fail");
    assert!(error.to_string().contains("qualification gate evidence"));
    assert!(error.to_string().contains("is stale"));
}

fn contract(extra: &str) -> Result<crate::Contract, ContractError> {
    let source = format!("{BASE}\n{extra}");
    let file = toml::from_str::<ContractFile>(&source)
        .map_err(|error| ContractError::one(error.to_string()))?;
    let mut state = MergeState::default();
    state.merge(file, "test")?;
    let contract = state.finish()?;
    validate_contract(&contract)?;
    Ok(contract)
}
