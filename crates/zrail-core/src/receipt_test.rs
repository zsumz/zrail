//! Strict receipt parsing and deterministic execution-context binding.

use super::*;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn accepts_a_versioned_receipt_and_binds_the_complete_reviewed_context() {
    let receipt = parse_execution_receipt(&receipt_json("runner 1.2.3", DIGEST, &execution()))
        .expect("valid receipt");
    assert_eq!(receipt.tests[0].status, ExecutionReceiptStatus::Passed);

    let mirror = mirror();
    let digest = test_mirror_input_sha256(
        &mirror,
        b"one",
        b"two",
        &[("Cargo.lock", b"lock"), ("Cargo.toml", b"manifest")],
    );
    assert_ne!(
        digest,
        test_mirror_input_sha256(
            &mirror,
            b"changed",
            b"two",
            &[("Cargo.lock", b"lock"), ("Cargo.toml", b"manifest")],
        )
    );
    assert_ne!(
        digest,
        test_mirror_input_sha256(
            &mirror,
            b"one",
            b"two",
            &[("Cargo.lock", b"changed"), ("Cargo.toml", b"manifest")],
        )
    );
    let mut changed = mirror;
    changed.execution.command.push_str(" --release");
    assert_ne!(
        digest,
        test_mirror_input_sha256(
            &changed,
            b"one",
            b"two",
            &[("Cargo.lock", b"lock"), ("Cargo.toml", b"manifest")],
        )
    );
}

#[test]
fn rejects_wrong_schema_unversioned_producers_and_invalid_execution() {
    let valid = execution();
    let mut unsorted = valid.clone();
    unsorted.features = vec!["z".into(), "a".into()];
    for source in [
        receipt_json("runner 1.2.3", DIGEST, &valid).replace("\"schema\":2", "\"schema\":1"),
        receipt_json("runner", DIGEST, &valid),
        receipt_json("runner 1.2.3", "ABC", &valid),
        receipt_json("runner 1.2.3", DIGEST, &unsorted),
        receipt_json("runner 1.2.3", DIGEST, &valid).replace(
            "\"tests\":[{\"id\":\"works\",\"status\":\"passed\"}]",
            "\"tests\":[{\"id\":\"works\",\"status\":\"passed\"},{\"id\":\"works\",\"status\":\"failed\"}]",
        ),
    ] {
        assert!(parse_execution_receipt(&source).is_err(), "{source}");
    }
}

fn mirror() -> TestMirrorContract {
    TestMirrorContract {
        production: "src/state.rs".into(),
        test: "tests/state.rs".into(),
        name: "works".into(),
        receipt: "evidence/state.json".into(),
        inputs: vec!["Cargo.lock".into(), "Cargo.toml".into()],
        execution: execution(),
        reason: "Exact behavior".into(),
    }
}

fn execution() -> TestExecutionIdentity {
    TestExecutionIdentity {
        command: "cargo test --package state --test state works --target x86_64-unknown-linux-gnu"
            .into(),
        package: "state".into(),
        default_features: false,
        features: vec!["strict".into()],
        target: "x86_64-unknown-linux-gnu".into(),
        toolchain: "rustc 1.90.0 (example 2026-01-01)".into(),
    }
}

fn receipt_json(producer: &str, input_sha256: &str, execution: &TestExecutionIdentity) -> String {
    format!(
        concat!(
            "{{\"schema\":2,\"producer\":\"{}\",\"input_sha256\":\"{}\",",
            "\"execution\":{},\"tests\":[{{\"id\":\"works\",\"status\":\"passed\"}}]}}",
        ),
        producer,
        input_sha256,
        serde_json::to_string(execution).expect("serialize execution")
    )
}
