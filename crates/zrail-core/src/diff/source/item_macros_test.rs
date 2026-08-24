//! Item-macro selectors and provenance participate in semantic diffs.

use crate::{ChangeKind, ItemMacroContract, MacroBindingMode};

use super::super::super::compare_architecture;
use crate::diff::compare_fixture_test::contract_with_hard_limit;

#[test]
fn adding_scoped_authority_is_a_grant() {
    let before = contract_with_hard_limit(300);
    let mut after = before.clone();
    after.source.rust.item_macros.push(ItemMacroContract {
        name: "criterion_group".into(),
        path: None,
        within: vec!["benches/**".into()],
        binding: None,
        source: None,
        manifest: None,
        reason: "Reviewed benchmark harness.".into(),
    });

    let report = compare_architecture(&before, None, &after, None);

    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Grant
            && change.rail == "rust.source-graph.item-macro"
            && change.subject.contains("within=benches/**")
    }));
}

#[test]
fn provenance_binding_changes_cannot_disappear_from_the_diff() {
    let mut before = contract_with_hard_limit(300);
    before.source.rust.item_macros.push(ItemMacroContract {
        name: "items".into(),
        path: None,
        within: Vec::new(),
        binding: None,
        source: None,
        manifest: None,
        reason: "Reviewed item generator.".into(),
    });
    let mut after = before.clone();
    after.source.rust.item_macros[0].binding = Some(MacroBindingMode::Exact);

    let report = compare_architecture(&before, None, &after, None);

    assert_eq!(report.summary.grants, 0);
    assert_eq!(report.summary.revokes, 1);
}
