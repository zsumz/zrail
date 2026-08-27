//! External item shape cannot be inferred from a local extension trait.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::*;

#[test]
fn external_variant_beats_out_of_scope_extension_trait() {
    assert_external_unresolved("let _ = external::Choice::Ready(44);");
}

#[test]
fn external_variant_beats_in_scope_extension_trait() {
    assert_external_unresolved(
        "use crate::traits::Extension; let _ = external::Choice::Ready(44);",
    );
}

#[test]
fn external_unit_variant_beats_extension_trait_const() {
    let mut index = external_fixture(
        "pub trait Extension { const Idle: external::Choice; }",
        "impl crate::traits::Extension for external::Choice { const Idle: external::Choice = loop {}; }",
        "use crate::traits::Extension; let _ = external::Choice::Idle;",
    );
    canonicalize_external(&mut index);
    let operations = external_owned(&index, "external::Choice::Idle");
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn fully_qualified_extension_trait_call_is_value() {
    let mut index = external_fixture(
        trait_declaration(),
        implementation(),
        "let _ = <external::Choice as crate::traits::Extension>::Ready(44);",
    );
    canonicalize_external(&mut index);
    assert!(
        external_owned(&index, "external::Choice::Ready").is_empty(),
        "operations: {:#?}",
        index
            .files
            .iter()
            .find(|file| file.relative == "src/user.rs")
            .expect("user file")
            .operations
    );
}

#[test]
fn fully_qualified_trait_alias_is_value() {
    let mut index = external_fixture(
        trait_declaration(),
        implementation(),
        "use crate::traits::Extension as Alias; let _ = <external::Choice as Alias>::Ready(44);",
    );
    canonicalize_external(&mut index);
    assert!(external_owned(&index, "external::Choice::Ready").is_empty());
}

#[test]
fn unresolved_qualified_trait_fails_closed() {
    let mut index = external_fixture(
        trait_declaration(),
        implementation(),
        "let _ = <external::Choice as Extension>::Ready(44);",
    );
    canonicalize_external(&mut index);
    let operations = external_owned(&index, "external::Choice::Ready");
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn external_unknown_item_without_variant_remains_unresolved() {
    let mut index = external_fixture("", "", "let _ = external::Choice::unknown(44);");
    canonicalize_external(&mut index);
    let operations = external_owned(&index, "external::Choice::unknown");
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn local_variant_collision_remains_constructor() {
    let source = "trait Extension { #[allow(non_snake_case)] fn Ready(_: u64) -> Self; } enum Choice { Ready(u64) } impl Extension for Choice { fn Ready(_: u64) -> Self { loop {} } } fn run() { let _ = Choice::Ready(44); }";
    let mut index = SourceIndex {
        files: vec![parsed_file("src/lib.rs", source)],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    };
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = matching_operations(
        &index,
        "src/lib.rs",
        OwnerKind::TypeConstruction,
        "crate::Choice::Ready",
    );
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn assert_external_unresolved(call: &str) {
    let mut index = external_fixture(trait_declaration(), implementation(), call);
    canonicalize_external(&mut index);
    let operations = external_owned(&index, "external::Choice::Ready");
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Unresolved);
}

fn canonicalize_external(index: &mut SourceIndex) {
    let compilation = domain();
    let modules = modules_for_domain(index, &compilation);
    let findings = canonicalize_operations_with_external(index, &compilation, &modules, "external");
    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
}

fn external_owned(index: &SourceIndex, selector: &str) -> Vec<crate::source::SourceOperationFact> {
    matching_operations(index, "src/user.rs", OwnerKind::TypeConstruction, selector)
}

fn external_fixture(traits: &str, implementation: &str, call: &str) -> SourceIndex {
    trait_fixture(traits, implementation, call)
}

fn trait_declaration() -> &'static str {
    "pub trait Extension { #[allow(non_snake_case)] fn Ready(value: u64) -> Self; }"
}

fn implementation() -> &'static str {
    "impl crate::traits::Extension for external::Choice { fn Ready(_: u64) -> Self { loop {} } }"
}
