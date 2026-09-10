//! Trusted assertion-discovery census; a candidate is never a verified migration assertion.

#[path = "rc9_inventory/collector.rs"]
mod collector;
#[path = "rc9_inventory/inputs.rs"]
mod inputs;
#[path = "rc9_inventory/inputs_test.rs"]
mod inputs_test;
#[path = "rc9_inventory/snapshot.rs"]
mod snapshot;

use std::{collections::BTreeSet, fs, io::Write, path::Path};
use syn::visit::Visit;

#[derive(serde::Serialize)]
struct Census<'a> {
    schema: u32,
    status: &'static str,
    assertion_inventory_complete: bool,
    snapshots: serde_json::Value,
    additional_rust_inputs_sha256: String,
    candidate_count: usize,
    files: &'a [snapshot::FileRecord],
    limitations: [&'static str; 4],
}

#[test]
fn census_distinguishes_code_assertions_helpers_and_opaque_inputs() {
    let source = r#"
        // assert!(false);
        #[test] fn detector() {
            let text = "assert!(false)";
            assert_eq!(actual, expected);
            assert_policy(&input);
            failures.push("missing property");
            opaque! { assert!(inside_macro) }
        }
    "#;
    let mut collector = collector::Collector::default();
    collector.visit_file(&syn::parse_file(source).expect("parse census fixture"));
    assert_eq!(
        collector
            .candidates
            .iter()
            .map(|candidate| candidate.kind)
            .collect::<Vec<_>>(),
        [
            "assertion-macro",
            "helper-call-candidate",
            "accumulator-candidate",
            "macro-review-boundary"
        ]
    );
    assert_eq!(collector.functions.len(), 1);
    assert_eq!(collector.functions[0].name, "detector");
    assert!(
        collector
            .candidates
            .iter()
            .all(|candidate| candidate.function == ["detector"])
    );
}

#[test]
fn census_locates_each_chained_failure_and_qualified_invocation() {
    let source = r#"
        fn guard(value: Value) {
            value.unwrap().expect("inner").unwrap_err().expect_err("outer");
            Option::unwrap(value);
            <Result<T, E>>::expect(value, "required");
            value.r#unwrap();
            let reference = Option::unwrap;
            let text = "value.expect(\"ignored\")";
            // value.unwrap();
            opaque! { value.unwrap() }
        }
    "#;
    let mut collector = collector::Collector::default();
    collector.prefix = "fixture:src/guard.rs".into();
    collector.visit_file(&syn::parse_file(source).expect("parse census fixture"));
    let failures = collector
        .candidates
        .iter()
        .filter(|candidate| candidate.kind == "fallible-call-candidate")
        .collect::<Vec<_>>();
    assert_eq!(failures.len(), 7);
    assert_eq!(
        failures
            .iter()
            .map(|candidate| &candidate.id)
            .collect::<BTreeSet<_>>()
            .len(),
        7,
    );
    let lines = source.lines().collect::<Vec<_>>();
    for candidate in failures {
        let identifier = &lines[candidate.line - 1][candidate.column - 1..candidate.end_column - 1];
        assert!(matches!(
            identifier,
            "unwrap" | "expect" | "unwrap_err" | "expect_err" | "r#unwrap"
        ));
        assert_eq!(candidate.function, ["guard"]);
    }
    assert_eq!(collector.candidates.len(), 8);
}

#[test]
fn census_binds_failure_preconditions_to_default_trait_functions() {
    let source = "trait Detector { fn validate(&self) { self.input().unwrap(); } }";
    let mut collector = collector::Collector::default();
    collector.visit_file(&syn::parse_file(source).expect("parse default helper"));
    assert_eq!(collector.functions.len(), 1);
    assert_eq!(collector.functions[0].name, "validate");
    assert_eq!(collector.candidates.len(), 1);
    assert_eq!(collector.candidates[0].kind, "fallible-call-candidate");
    assert_eq!(collector.candidates[0].function, ["validate"]);
}

#[test]
#[ignore = "requires explicitly prefetched frozen consumer checkouts and an output path"]
fn write_frozen_assertion_census() {
    let roots = std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("set ZRAIL_RC9_SNAPSHOTS");
    let output = std::env::var_os("ZRAIL_RC9_CENSUS").expect("set ZRAIL_RC9_CENSUS");
    let testkit = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates directory")
        .join("zrail-testkit/tests/fixtures/rc9");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(testkit.join("snapshots.json")).expect("read pins"))
            .expect("parse pins");
    assert_eq!(manifest["schema"], 1);
    let (additional, additional_rust_inputs_sha256) = inputs::AdditionalInputs::read(&testkit);
    let mut files = Vec::new();
    for pin in manifest["consumers"].as_array().expect("consumer pins") {
        let root = Path::new(&roots).join(pin["name"].as_str().expect("snapshot directory"));
        files.extend(snapshot::inspect(&root, pin, &additional));
    }
    additional.verify(&files);
    let mut ids = BTreeSet::new();
    for candidate in files.iter().flat_map(|file| &file.candidates) {
        assert!(
            ids.insert(&candidate.id),
            "duplicate migration-candidate identity"
        );
    }
    let census = serde_json::to_value(Census {
        schema: 1, status: "discovery-only", assertion_inventory_complete: false,
        snapshots: manifest, additional_rust_inputs_sha256, candidate_count: ids.len(), files: &files,
        limitations: [
            "Every candidate and non-Rust file requires assertion-level review.",
            "Helper results, registry instantiations, raw-text predicates, and failure branches require manual expansion.",
            "Opaque macro bodies and malformed Rust remain explicit review boundaries.",
            "No candidate is assigned a behavioral or replacement disposition by this census."
        ]
    }).expect("normalize census field order");
    let bytes = serde_json::to_vec(&census).expect("serialize census");
    assert!(
        bytes.len() <= 64 * 1024 * 1024,
        "census exceeds the 64 MiB audit input bound"
    );
    std::io::stdout()
        .write_all(
            format!(
                "{} tracked files, {} candidates; sha256:{}",
                files.len(),
                ids.len(),
                zrail_core::sha256_hex(&bytes)
            )
            .as_bytes(),
        )
        .expect("write census summary");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .expect("fresh census output")
        .write_all(&bytes)
        .expect("write trusted discovery evidence");
}
