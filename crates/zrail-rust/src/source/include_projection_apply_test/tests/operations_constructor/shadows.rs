//! Block bindings select construction identity before declaration-shape lookup.

use zrail_core::OwnerKind;

use super::super::{
    canonicalize_operation_worlds, canonicalize_operations, domain, matching_operations,
    parsed_file,
};
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn block_type_alias_shadows_outer_type() {
    assert_target(
        "struct A { value: u64 } struct B { value: u64 } fn build() { type A = B; let _ = A { value: 44 }; }",
        "crate::B",
    );
}

#[test]
fn block_use_alias_shadows_outer_type() {
    assert_target(
        "struct A { value: u64 } struct B { value: u64 } fn build() { use crate::B as A; let _ = A { value: 44 }; }",
        "crate::B",
    );
}

#[test]
fn block_tuple_struct_is_not_suppressed_by_outer_named_struct() {
    assert_block_local("struct A { value: u64 } fn build() { struct A(u64); let _ = A(44); }");
}

#[test]
fn block_enum_variant_is_not_suppressed_by_outer_variant_shape() {
    assert_block_local(
        "enum A { Variant { value: u64 } } fn build() { enum A { Variant(u64) } let _ = A::Variant(44); }",
    );
}

#[test]
fn block_union_does_not_impersonate_outer_struct() {
    assert_block_local(
        "struct A { value: u64 } fn build() { union A { value: u64 } let _ = A { value: 44 }; }",
    );
}

#[test]
fn cfg_partitioned_block_aliases_never_select_outer_type() {
    let mut left = domain();
    left.feature_world = Some("left".into());
    left.active_features.insert("left".into());
    let mut right = domain();
    right.feature_world = Some("right".into());
    let mut index = fixture(
        r#"struct A { value: u64 }
struct Left { value: u64 }
struct Right { value: u64 }
fn build() {
    #[cfg(feature = "left")] type A = Left;
    #[cfg(not(feature = "left"))] type A = Right;
    let _ = A { value: 44 };
}"#,
    );
    let findings = canonicalize_operation_worlds(&mut index, &[left, right], &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    for selector in ["crate::Left", "crate::Right"] {
        assert_eq!(matching(&index, selector).len(), 1, "owner {selector}");
    }
    assert!(matching(&index, "crate::A").is_empty());
}

fn assert_target(source: &str, target: &str) {
    let mut index = fixture(source);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(matching(&index, target).len(), 1, "owner {target}");
    assert!(matching(&index, "crate::A").is_empty());
}

fn assert_block_local(source: &str) {
    let mut index = fixture(source);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(matching(&index, "crate::A").is_empty());
    assert!(index.files[0].operations.iter().any(|operation| {
        operation.kind == SourceOperationKind::TypeConstruction
            && operation.identity.quality == zrail_core::AnalysisQuality::Exact
    }));
}

fn matching(index: &SourceIndex, selector: &str) -> Vec<crate::source::SourceOperationFact> {
    matching_operations(index, "src/lib.rs", OwnerKind::TypeConstruction, selector)
}

fn fixture(source: &str) -> SourceIndex {
    SourceIndex {
        files: vec![parsed_file("src/lib.rs", source)],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
