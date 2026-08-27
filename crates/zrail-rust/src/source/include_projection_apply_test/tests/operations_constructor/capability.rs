//! Constructor values are governed when construction authority is acquired.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::super::{canonicalize_operations, domain, matching_operations, parsed_file};
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn tuple_constructor_stored_then_called_reaches_owner() {
    assert_capability("let make = Ticket; let _ = make(44);", "crate::Ticket");
}

#[test]
fn variant_constructor_stored_then_called_reaches_owner() {
    assert_capability(
        "let make = Choice::Ready; let _ = make(44);",
        "crate::Choice::Ready",
    );
}

#[test]
fn constructor_passed_to_function_reaches_owner() {
    assert_capability("let _ = apply(Ticket, 44);", "crate::Ticket");
}

#[test]
fn constructor_passed_to_iterator_adapter_reaches_owner() {
    assert_capability("let _ = Option::map(Some(44), Ticket);", "crate::Ticket");
}

#[test]
fn constructor_cast_to_fn_pointer_reaches_owner() {
    assert_capability(
        "let _ = (Ticket as fn(u64) -> Ticket)(44);",
        "crate::Ticket",
    );
}

#[test]
fn constructor_returned_from_function_reaches_owner() {
    let mut index = fixture("fn constructor() -> fn(u64) -> Ticket { Ticket }");
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_one_capability(&index, "crate::Ticket");
}

#[test]
fn ordinary_function_value_does_not_reach_constructor_owner() {
    let mut index = fixture(
        "fn convert(value: u64) -> u64 { value } fn use_it() { let f = convert; let _ = f(44); }",
    );
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(
        matching_operations(
            &index,
            "src/lib.rs",
            OwnerKind::TypeConstruction,
            "crate::Ticket",
        )
        .is_empty()
    );
    assert!(
        index.files[0]
            .operations
            .iter()
            .all(|operation| operation.kind != SourceOperationKind::ConstructorCapability)
    );
}

#[test]
fn direct_constructor_call_does_not_duplicate_capability() {
    let mut index = fixture("fn build() { let _ = Ticket(44); }");
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = matching_operations(
        &index,
        "src/lib.rs",
        OwnerKind::TypeConstruction,
        "crate::Ticket",
    );
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].kind, SourceOperationKind::TypeConstruction);
}

fn assert_capability(body: &str, selector: &str) {
    let mut index = fixture(&format!("fn build() {{ {body} }}"));
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_one_capability(&index, selector);
}

fn assert_one_capability(index: &SourceIndex, selector: &str) {
    let operations =
        matching_operations(index, "src/lib.rs", OwnerKind::TypeConstruction, selector);
    assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
    assert_eq!(
        operations[0].kind,
        SourceOperationKind::ConstructorCapability
    );
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn fixture(body: &str) -> SourceIndex {
    let source = format!(
        "struct Ticket(u64); enum Choice {{ Ready(u64) }} fn apply<T, U>(f: fn(T) -> U, value: T) -> U {{ f(value) }} {body}"
    );
    SourceIndex {
        files: vec![parsed_file("src/lib.rs", &source)],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
