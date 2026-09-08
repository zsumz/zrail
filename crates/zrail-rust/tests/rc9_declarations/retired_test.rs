//! Frozen release-graph bundle qualifies syntax, raw tokens, presence and physical trees together.

#[path = "retired_boundary_test.rs"]
mod boundary;
#[path = "retired_cases.rs"]
mod cases;
use super::declaration_legacy as legacy;
#[path = "retired_model.rs"]
mod model;
#[path = "retired_legacy.rs"]
mod physical;
#[path = "retired_qualification.rs"]
mod qualification;

#[test]
fn retired_bundle_preserves_original_predicates_and_required_inputs() {
    let fixture = model::Fixture::new();
    let rows = cases::run(&fixture, &model::policies());
    assert_eq!(rows.len(), 144);
}

#[test]
fn retired_physical_oracles_retain_the_complete_original_function_bodies() {
    let root = model::project();
    let source = std::fs::read_to_string(
        root.join("crates/zrail-testkit/tests/fixtures/rc9/declarations/release_graph.rs"),
    )
    .expect("frozen source");
    assert_eq!(
        zrail_core::sha256_hex(source.as_bytes()),
        "e30e48c6f261690844c9db7ef14d02316740c4b4d8eec322241268d454c947ff"
    );
    let harness = std::fs::read_to_string(
        root.join("crates/zrail-rust/tests/rc9_declarations/retired_legacy.rs"),
    )
    .expect("harness");
    for (start, end) in [(49, 60), (81, 84)] {
        let excerpt = source
            .lines()
            .skip(start - 1)
            .take(end - start + 1)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            harness.contains(&excerpt),
            "unmodified original lines {start}..{end}"
        );
    }
}

#[test]
#[ignore = "requires prefetched snapshots, committed inputs and fresh ZRAIL_RC9_RETIRED_REPORT"]
fn qualify_frozen_retired_release_graph_bundle() {
    qualification::run();
}
