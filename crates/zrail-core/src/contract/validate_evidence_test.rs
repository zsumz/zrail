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
fn accepts_one_exact_reasoned_test_mirror_pair() {
    let extra = r#"
[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "src/state_test.rs"
name = "state_transitions"
receipt = "evidence/state.json"
inputs = ["Cargo.lock", "Cargo.toml"]
command = "cargo test --package state state_transitions --target x86_64-unknown-linux-gnu"
package = "state"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
reason = "State transitions are exercised through the public surface."
"#;

    assert!(contract(extra).is_ok());
}

#[test]
fn rejects_reused_or_invalid_test_mirror_identities() {
    let extra = r#"
[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "src/state_test.rs"
name = "not::exact"
receipt = "evidence/state.json"
inputs = ["Cargo.lock", "Cargo.toml"]
command = "cargo test --package state not_exact --target x86_64-unknown-linux-gnu"
package = "state"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
reason = "State behavior."

[[source.rust.test_mirrors]]
production = "src/other.rs"
test = "src/state_test.rs"
name = "other_behavior"
receipt = "evidence/state.json"
inputs = ["Cargo.lock", "Cargo.toml"]
command = "cargo test --package state other_behavior --target x86_64-unknown-linux-gnu"
package = "state"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
reason = "Other behavior."

[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "src/third.txt"
name = "third_behavior"
receipt = "evidence/third.txt"
inputs = ["Cargo.lock", "Cargo.toml"]
command = "cargo test --package state third_behavior --target x86_64-unknown-linux-gnu"
package = "state"
default_features = true
features = []
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
reason = "Third behavior."
"#;

    let message = contract(extra)
        .expect_err("invalid mirrors must fail")
        .to_string();
    assert!(message.contains("reuses production path"));
    assert!(message.contains("reuses test path"));
    assert!(message.contains("reuses receipt path"));
    assert!(message.contains("invalid exact test name"));
    assert!(message.contains("must name a .rs file"));
    assert!(message.contains("must name a .json file"));
}

#[test]
fn rejects_incomplete_or_ambiguous_test_execution_context() {
    let extra = r#"
[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "src/state_test.rs"
name = "state_transitions"
receipt = "evidence/state.json"
inputs = ["Cargo.toml", "Cargo.toml", "zrail.lock"]
command = " cargo test"
package = "bad package"
default_features = true
features = ["z", "a"]
target = ""
toolchain = "1.90.0\nchanged"
reason = "State transitions are exercised through the public surface."
"#;

    let message = contract(extra)
        .expect_err("incomplete execution context must fail")
        .to_string();
    assert!(message.contains("must bind \"Cargo.lock\""));
    assert!(message.contains("repeats input path"));
    assert!(message.contains("invalid test mirror input"));
    assert!(message.contains("execution package is invalid"));
    assert!(message.contains("features must be valid, unique, and sorted"));
    assert!(message.contains("execution command"));
    assert!(message.contains("execution target"));
    assert!(message.contains("execution toolchain"));
}

#[test]
fn accepts_a_connected_local_and_ci_graph() {
    let extra = r#"
[[gate]]
name = "check"
kind = "local"
path = "scripts/check"
inputs = ["scripts/structure-check", "scripts/package-check"]
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
fn rejects_unsafe_or_ambiguous_gate_inputs() {
    let extra = r#"
[[gate]]
name = "check"
kind = "local"
path = "scripts/check"
inputs = ["scripts/check", "scripts/helper", "scripts/helper", "zrail.lock"]
reason = "Canonical local qualification."

[[invariant]]
id = "ARCH-01"
title = "Architecture is qualified"
status = "enforced"
document = "docs/architecture.md#arch-01"
evidence = ["rust-test:src/architecture_test.rs::qualifies", "gate:check"]
"#;

    let error = contract(extra).expect_err("invalid gate inputs must fail");
    let message = error.to_string();
    assert!(message.contains("repeats its primary path"));
    assert!(message.contains("duplicate input"));
    assert!(message.contains("cannot attest its own contents"));
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
