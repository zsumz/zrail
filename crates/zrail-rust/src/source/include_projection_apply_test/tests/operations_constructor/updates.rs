//! Functional updates resolve their declaration after lexical shadow selection.

use zrail_core::OwnerKind;

use super::super::{canonicalize_operation_worlds, canonicalize_operations, domain, parsed_file};
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn block_type_alias_update_reads_underlying_fields() {
    assert_alias_update("type A = B;", "crate::B::real");
}

#[test]
fn block_use_alias_update_reads_underlying_fields() {
    assert_alias_update("use crate::B as A;", "crate::B::real");
}

#[test]
fn block_struct_update_does_not_borrow_outer_fields() {
    let mut index = fixture(
        r"struct A { outer: u64 }
fn update() {
    struct A { inner: u64 }
    let base = A { inner: 1 };
    let _ = A { ..base };
}",
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_no_outer_field(&index);
    assert!(field_reads(&index).iter().any(|operation| {
        operation.identity.name.contains("<block@src/lib.rs:")
            && operation.identity.name.ends_with("::A::inner")
    }));
}

#[test]
fn block_union_update_fails_closed_without_outer_fields() {
    let mut index = fixture(
        r"struct A { outer: u64 }
fn update() {
    union A { inner: u64 }
    let base = A { inner: 1 };
    let _ = A { ..base };
}",
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_no_outer_field(&index);
    assert!(field_reads(&index).iter().any(|operation| {
        operation.identity.name.contains("<block@src/lib.rs:")
            && operation.identity.name.ends_with("::A::*")
            && operation.identity.quality == zrail_core::AnalysisQuality::Unresolved
    }));
}

#[test]
fn cfg_partitioned_block_alias_updates_read_each_world() {
    let mut left = domain();
    left.feature_world = Some("left".into());
    left.active_features.insert("left".into());
    let mut right = domain();
    right.feature_world = Some("right".into());
    let mut index = fixture(
        r#"struct A { outer: u64 }
struct Left { left: u64 }
struct Right { right: u64 }
fn update() {
    #[cfg(feature = "left")] type A = Left;
    #[cfg(not(feature = "left"))] type A = Right;
    let _ = A { ..todo!() };
}"#,
    );
    let findings = canonicalize_operation_worlds(&mut index, &[left, right], &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_no_outer_field(&index);
    for selector in ["crate::Left::left", "crate::Right::right"] {
        assert_eq!(matching(&index, selector).len(), 1, "owner {selector}");
    }
}

#[test]
fn cfg_partitioned_named_and_union_updates_keep_opaque_world() {
    let mut left = domain();
    left.feature_world = Some("left".into());
    left.active_features.insert("left".into());
    let mut right = domain();
    right.feature_world = Some("right".into());
    let mut index = fixture(
        r#"struct Outer { outer: u64 }
struct Left { left: u64 }
fn update() {
    #[cfg(feature = "left")] type A = Left;
    #[cfg(not(feature = "left"))] union A { right: u64 }
    let _ = A { ..todo!() };
}"#,
    );
    let findings = canonicalize_operation_worlds(&mut index, &[left, right], &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(matching(&index, "crate::Left::left").len(), 2);
    assert!(field_reads(&index).iter().any(|operation| {
        operation.identity.name.contains("<block@src/lib.rs:")
            && operation.identity.name.ends_with("::A::*")
            && operation.identity.quality == zrail_core::AnalysisQuality::Unresolved
    }));
}

#[test]
fn deferred_update_subtracts_explicit_field_cfg() {
    let mut index = fixture(
        r#"struct State {
    public: u64,
    #[cfg(feature = "extra")]
    extra: u64,
}
fn update(base: State) {
    let _ = State {
        #[cfg(feature = "direct")]
        public: 1,
        ..base
    };
}"#,
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let public = matching(&index, "crate::State::public");
    assert_eq!(public.len(), 1, "public reads: {public:#?}");
    assert_eq!(
        public[0].identity.guard.canonical_name(),
        "cfg:not(feature=\"direct\")"
    );
    let extra = matching(&index, "crate::State::extra");
    assert_eq!(extra.len(), 1, "extra reads: {extra:#?}");
    assert_eq!(
        extra[0].identity.guard.canonical_name(),
        "cfg:feature=\"extra\""
    );
}

#[test]
fn deferred_update_unites_cfg_partitioned_fields() {
    let mut index = fixture(
        r#"struct State {
    #[cfg(feature = "wide")]
    value: u64,
    #[cfg(not(feature = "wide"))]
    value: u32,
}
fn update(base: State) { let _ = State { ..base }; }"#,
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let value = matching(&index, "crate::State::value");
    assert_eq!(value.len(), 1, "value reads: {value:#?}");
    assert_eq!(value[0].identity.guard.canonical_name(), "ordinary");
}

fn assert_alias_update(binding: &str, selector: &str) {
    let mut index = fixture(&format!(
        "struct A {{ outer: u64 }} struct B {{ real: u64 }} fn update(base: B) {{ {binding} let _ = A {{ ..base }}; }}"
    ));
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(matching(&index, selector).len(), 1, "owner {selector}");
    assert_no_outer_field(&index);
}

fn assert_no_outer_field(index: &SourceIndex) {
    assert!(
        !field_reads(index)
            .iter()
            .any(|operation| operation.identity.name == "crate::A::outer"),
        "unexpected outer field reads: {:#?}",
        index.files[0].operations
    );
}

fn matching(index: &SourceIndex, selector: &str) -> Vec<crate::source::SourceOperationFact> {
    super::super::matching_operations(index, "src/lib.rs", OwnerKind::FieldRead, selector)
}

fn field_reads(index: &SourceIndex) -> Vec<&crate::source::SourceOperationFact> {
    index.files[0]
        .operations
        .iter()
        .filter(|operation| operation.kind == SourceOperationKind::FieldRead)
        .collect()
}

fn fixture(source: &str) -> SourceIndex {
    SourceIndex {
        files: vec![parsed_file("src/lib.rs", source)],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
