//! Analysis-budget edits are explicit reviewable changes.

use super::super::compare_fixture_test::contract_with_hard_limit;
use super::*;

#[test]
fn adding_or_raising_an_override_grants_and_tightening_revokes() {
    let before = contract_with_hard_limit(300);
    let mut raised = before.clone();
    raised.analysis.limits.projected_facts = Some(500_000);

    let mut changes = Vec::new();
    compare(&before, &raised, &mut changes);
    assert_eq!(changes[0].kind, ChangeKind::Grant);

    let mut tightened = raised.clone();
    tightened.analysis.limits.projected_facts = Some(250_000);
    changes.clear();
    compare(&raised, &tightened, &mut changes);
    assert_eq!(changes[0].kind, ChangeKind::Revoke);
}
