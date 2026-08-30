//! Declaration occurrence facts share the transaction and retained-fact budget.

use super::{
    ProjectionLimits, bindings, fact_count, fact_lengths, fixture_index, projected_call_count,
};

#[test]
fn leaf_fact_exhaustion_rolls_back_earlier_path_projection() {
    let mut index = fixture_index();
    let syntax = syn::parse_file("struct Permit { epoch: u64 }").unwrap();
    index.files[0].type_policy = crate::source::type_policy_index::collect(&syntax).0;
    let bindings = bindings(&index);
    let before = fact_lengths(&index);
    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 1_000,
            projected_facts: 1,
        },
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("fact safety budget"));
    assert_eq!(before, fact_lengths(&index));
    assert_eq!(projected_call_count(&index), 0);
    assert!(
        index.files[0].type_policy.declarations[0]
            .module_occurrences
            .is_empty()
    );
}

#[test]
fn retained_declaration_occurrences_count_toward_the_per_file_limit() {
    let mut index = fixture_index();
    let syntax = syn::parse_file("struct Permit { epoch: u64 }").unwrap();
    index.files[0].type_policy = crate::source::type_policy_index::collect(&syntax).0;
    let bindings = bindings(&index);
    let before = index.files.iter().map(fact_count).sum::<usize>();
    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 1_000,
            projected_facts: 2,
        },
    );
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(
        index.files.iter().map(fact_count).sum::<usize>(),
        before + 1
    );
    assert_eq!(
        index.files[0].type_policy.declarations[0]
            .module_occurrences
            .len(),
        1
    );
}
