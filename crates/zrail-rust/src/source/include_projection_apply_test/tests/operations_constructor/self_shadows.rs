//! Impl `Self` resolves through the same lexical binding truth as constructions.

use zrail_core::OwnerKind;

use super::super::{
    canonicalize_operation_worlds, canonicalize_operations, domain, matching_operations,
    parsed_file,
};
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn block_local_struct_impl_self_reaches_local_constructor() {
    let mut index = fixture(
        r"struct A { outer: u64 }
fn scope() {
    struct A { inner: u64 }
    impl A { fn make() -> Self { Self { inner: 1 } } }
}",
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_local_self(&index, "::A");
    assert!(matching(&index, "crate::A").is_empty());
}

#[test]
fn block_local_enum_impl_self_reaches_local_variant() {
    let mut index = fixture(
        r"enum A { Outer }
fn scope() {
    enum A { Inner }
    impl A { fn make() -> Self { Self::Inner } }
}",
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_local_self(&index, "::A::Inner");
    assert!(matching(&index, "crate::A::Outer").is_empty());
}

#[test]
fn cfg_partitioned_block_impl_self_never_selects_outer_type() {
    let mut left = domain();
    left.feature_world = Some("left".into());
    left.active_features.insert("left".into());
    let mut right = domain();
    right.feature_world = Some("right".into());
    let mut index = fixture(
        r#"enum A { Outer }
fn scope() {
    #[cfg(feature = "left")] struct A { left: u64 }
    #[cfg(not(feature = "left"))] enum A { Right }
    #[cfg(feature = "left")]
    impl A { fn make() -> Self { Self { left: 1 } } }
    #[cfg(not(feature = "left"))]
    impl A { fn make() -> Self { Self::Right } }
}"#,
    );
    let findings = canonicalize_operation_worlds(&mut index, &[left, right], &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_local_self(&index, "::A");
    assert_local_self(&index, "::A::Right");
    assert!(matching(&index, "crate::A").is_empty());
    assert!(matching(&index, "crate::A::Outer").is_empty());
}

fn assert_local_self(index: &SourceIndex, suffix: &str) {
    assert!(index.files[0].operations.iter().any(|operation| {
        operation.kind == SourceOperationKind::TypeConstruction
            && operation.identity.name.contains("<block@src/lib.rs:")
            && operation.identity.name.ends_with(suffix)
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
