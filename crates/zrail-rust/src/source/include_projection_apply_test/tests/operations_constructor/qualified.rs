//! Qualified constructor spelling never collapses to an unrelated trailing value.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::super::{canonicalize_operations, domain, matching_operations, parsed_file};
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn type_relative_tuple_variant_reaches_owner() {
    assert_exact_construction("let _ = <Choice>::Ready(1);", "crate::Choice::Ready");
}

#[test]
fn type_relative_unit_variant_reaches_owner() {
    assert_exact_construction("let _ = <Choice>::Idle;", "crate::Choice::Idle");
}

#[test]
fn type_relative_constructor_capability_reaches_owner() {
    let index = canonicalized("let make = <Choice>::Ready; let _ = make(44);");
    let operations = owned(&index, "crate::Choice::Ready");
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(
        operations[0].kind,
        SourceOperationKind::ConstructorCapability
    );
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

#[test]
fn same_named_free_function_does_not_hide_variant() {
    assert_exact_construction(
        "#[allow(non_snake_case)] fn Ready(_: u64) {} let _ = <Choice>::Ready(1);",
        "crate::Choice::Ready",
    );
}

#[test]
fn same_named_free_constant_does_not_hide_variant() {
    assert_exact_construction(
        "#[allow(non_upper_case_globals)] const Idle: u64 = 0; let _ = <Choice>::Idle;",
        "crate::Choice::Idle",
    );
}

#[test]
fn qualified_associated_function_is_discarded_as_value() {
    let source = "struct Item; impl Item { fn make(_: u64) -> Self { loop {} } } fn run() { let _ = <Item>::make(1); }";
    let mut index = index(source);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(owned(&index, "crate::Item::make").is_empty());
}

#[test]
fn generic_qualified_constructor_fails_closed() {
    assert_unresolved("fn run<T>() { let _ = <T>::Ready(1); }", "<T>::Ready");
}

#[test]
fn type_relative_variant_ignores_local_value_shadow() {
    assert_exact_construction(
        "#[allow(non_snake_case)] let Ready = 0; let _ = <Choice>::Ready(1);",
        "crate::Choice::Ready",
    );
}

#[test]
fn trait_qualified_projection_fails_closed() {
    assert_unresolved(
        "trait Extension { type Associated; } fn run<T: Extension>() { let _ = <T as Extension>::Associated::Ready(1); }",
        "<T as Extension>::Associated::Ready",
    );
}

fn assert_exact_construction(body: &str, selector: &str) {
    let index = canonicalized(body);
    let operations = owned(&index, selector);
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].kind, SourceOperationKind::TypeConstruction);
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn assert_unresolved(source: &str, written: &str) {
    let mut index = index(source);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = &index.files[0].operations;
    let operation = operations
        .iter()
        .find(|operation| operation.identity.name == written)
        .unwrap_or_else(|| panic!("missing {written}: {operations:#?}"));
    assert_eq!(operation.identity.quality, AnalysisQuality::Unresolved);
}

fn canonicalized(body: &str) -> SourceIndex {
    let source = format!("enum Choice {{ Ready(u64), Idle }} fn run() {{ {body} }}");
    let mut index = index(&source);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);
    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    index
}

fn owned(index: &SourceIndex, selector: &str) -> Vec<crate::source::SourceOperationFact> {
    matching_operations(index, "src/lib.rs", OwnerKind::TypeConstruction, selector)
}

fn index(source: &str) -> SourceIndex {
    SourceIndex {
        files: vec![parsed_file("src/lib.rs", source)],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
