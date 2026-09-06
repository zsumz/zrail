//! Extra fragment discovery is exact, source-bound and independent of filename guesses.

use super::{inputs::AdditionalInputs, snapshot::FileRecord};

fn fixture() -> serde_json::Value {
    serde_json::json!({"schema": 1, "inputs": [{
        "repository": "fixture/repository", "commit": "a".repeat(40),
        "path": "tests/helper.inc", "sha256": "b".repeat(64), "syntax": "items",
        "reason": "Reviewed Rust fixture input.",
        "review_sources": [{"path": "tests/owner.rs", "sha256": "c".repeat(64)}]
    }]})
}

fn parse(value: &serde_json::Value) -> AdditionalInputs {
    AdditionalInputs::parse(&serde_json::to_vec(value).expect("fixture registry"))
}

fn file(path: &str, sha256: &str) -> FileRecord {
    FileRecord {
        repository: "fixture/repository".into(),
        commit: "a".repeat(40),
        path: path.into(),
        mode: "100644".into(),
        git_object: "d".repeat(40),
        sha256: Some(sha256.into()),
        parse_error: None,
        functions: vec![],
        candidates: vec![],
        review: "pending",
    }
}

#[test]
fn additional_rust_inputs_require_exact_source_and_review_identities() {
    let inputs = parse(&fixture());
    assert!(inputs.selected(
        "fixture/repository",
        &"a".repeat(40),
        "tests/helper.inc",
        &"b".repeat(64)
    ));
    assert!(!inputs.selected(
        "fixture/repository",
        &"a".repeat(40),
        "other/helper.inc",
        &"b".repeat(64)
    ));
    for (commit, hash) in [
        ("d".repeat(40), "b".repeat(64)),
        ("a".repeat(40), "d".repeat(64)),
    ] {
        assert!(
            std::panic::catch_unwind(|| inputs.selected(
                "fixture/repository",
                &commit,
                "tests/helper.inc",
                &hash
            ))
            .is_err()
        );
    }
    let mut files = vec![
        file("tests/helper.inc", &"b".repeat(64)),
        file("tests/owner.rs", &"c".repeat(64)),
    ];
    inputs.verify(&files);
    files[1].sha256 = Some("e".repeat(64));
    assert!(std::panic::catch_unwind(|| inputs.verify(&files)).is_err());
    files.pop();
    assert!(std::panic::catch_unwind(|| inputs.verify(&files)).is_err());
    assert!(std::panic::catch_unwind(|| inputs.verify(&[])).is_err());
}

#[test]
fn extra_fragment_registry_rejects_ambiguous_or_unreviewed_authority() {
    for mutation in [
        "schema",
        "duplicate",
        "unknown",
        "syntax",
        "blank-reason",
        "hash",
        "parent",
        "absolute",
        "glob",
        "ordinary-rust",
        "missing-review",
        "duplicate-review",
    ] {
        let mut value = fixture();
        let entry = value["inputs"][0].clone();
        match mutation {
            "schema" => value["schema"] = 2.into(),
            "duplicate" => value["inputs"].as_array_mut().expect("inputs").push(entry),
            "unknown" => value["inputs"][0]["accept_unresolved"] = true.into(),
            "syntax" => value["inputs"][0]["syntax"] = "expressions".into(),
            "blank-reason" => value["inputs"][0]["reason"] = " ".into(),
            "hash" => value["inputs"][0]["sha256"] = "b".repeat(63).into(),
            "parent" => value["inputs"][0]["path"] = "../helper.inc".into(),
            "absolute" => value["inputs"][0]["path"] = "/helper.inc".into(),
            "glob" => value["inputs"][0]["path"] = "tests/*.inc".into(),
            "ordinary-rust" => value["inputs"][0]["path"] = "tests/helper.rs".into(),
            "missing-review" => value["inputs"][0]["review_sources"] = serde_json::json!([]),
            "duplicate-review" => value["inputs"][0]["review_sources"]
                .as_array_mut()
                .expect("sources")
                .push(entry["review_sources"][0].clone()),
            _ => unreachable!(),
        }
        assert!(
            std::panic::catch_unwind(|| parse(&value)).is_err(),
            "{mutation}"
        );
    }
}
