//! Exact evidence for the fixture invariant.

#[test]
fn qualification_graph_is_live() {
    assert_eq!(super::worker::value(), 5);
}
