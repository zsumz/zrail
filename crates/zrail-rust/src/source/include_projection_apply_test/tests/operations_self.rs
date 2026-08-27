//! `Self` keeps the resolved implementation subject across physical source files.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::*;
use crate::source::SourceIndex;

#[test]
fn cross_file_self_named_construction_reaches_owner() {
    assert_self_owner(
        "pub struct State { pub epoch: u64 }",
        "impl crate::state::State { fn mint() -> Self { Self { epoch: 0 } } }",
        OwnerKind::TypeConstruction,
        "crate::state::State",
    );
}

#[test]
fn cross_file_self_tuple_construction_reaches_owner() {
    assert_self_owner(
        "pub struct Tuple(pub u64);",
        "impl crate::state::Tuple { fn mint() -> Self { Self(0) } }",
        OwnerKind::TypeConstruction,
        "crate::state::Tuple",
    );
}

#[test]
fn cross_file_self_tuple_variant_reaches_owner() {
    assert_self_owner(
        "pub enum Choice { Tuple(u64) }",
        "impl crate::state::Choice { fn mint() -> Self { Self::Tuple(0) } }",
        OwnerKind::TypeConstruction,
        "crate::state::Choice::Tuple",
    );
}

#[test]
fn cross_file_self_record_variant_reaches_owner() {
    assert_self_owner(
        "pub enum Choice { Record { value: u64 } }",
        "impl crate::state::Choice { fn mint() -> Self { Self::Record { value: 0 } } }",
        OwnerKind::TypeConstruction,
        "crate::state::Choice::Record",
    );
}

#[test]
fn cross_file_self_unit_variant_reaches_owner() {
    assert_self_owner(
        "pub enum Choice { Unit }",
        "impl crate::state::Choice { fn mint() -> Self { Self::Unit } }",
        OwnerKind::TypeConstruction,
        "crate::state::Choice::Unit",
    );
}

#[test]
fn cross_file_self_functional_update_uses_declared_fields() {
    assert_self_owner(
        "pub struct State { pub epoch: u64, pub secret: u64 }",
        r"impl crate::state::State {
    fn update(previous: Self) -> Self { Self { epoch: 1, ..previous } }
}",
        OwnerKind::FieldRead,
        "crate::state::State::secret",
    );
}

fn assert_self_owner(model: &str, implementation: &str, kind: OwnerKind, selector: &str) {
    let root = parsed_file("src/lib.rs", &format!("mod state;\n{implementation}"));
    let state = parsed_file("src/state.rs", model);
    let compilation = domain();
    let modules = [module_edge(
        "src/lib.rs",
        "state",
        "src/state.rs",
        module(&root.modules, "state"),
        &compilation,
    )];
    let mut index = index([root, state]);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = matching_operations(&index, "src/lib.rs", kind, selector);
    assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
