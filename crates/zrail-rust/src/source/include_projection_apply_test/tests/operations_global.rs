//! Rooted operation paths retain their edition-sensitive namespace origin.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::*;
use crate::source::SourceIndex;

#[test]
fn edition_2024_global_path_stays_distinct_from_local_module() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"mod wire { pub struct Ticket { pub id: u64 } }
fn external() { let _ = ::wire::Ticket { id: 1 }; }
fn local() { let _ = wire::Ticket { id: 2 }; }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let constructions = index.files[0]
        .operations
        .iter()
        .filter(|operation| operation.kind == crate::source::SourceOperationKind::TypeConstruction)
        .collect::<Vec<_>>();
    assert_eq!(constructions.len(), 2, "constructions: {constructions:#?}");
    let external = constructions
        .iter()
        .find(|operation| operation.identity.written.as_deref() == Some("::wire::Ticket"))
        .expect("rooted construction");
    assert!(external.identity.canonical.is_empty());
    let local = constructions
        .iter()
        .find(|operation| operation.identity.written.as_deref() == Some("wire::Ticket"))
        .expect("local construction");
    assert_eq!(local.identity.canonical, ["wire::Ticket"]);
}

#[test]
fn edition_2015_global_path_uses_crate_root() {
    let mut compilation = domain();
    compilation.edition = "2015".into();
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"mod wire { pub struct Ticket { pub id: u64 } }
fn mint() { let _ = ::wire::Ticket { id: 1 }; }",
    )]);
    let findings = canonicalize_operations(&mut index, &compilation, &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = matching_operations(
        &index,
        "src/lib.rs",
        OwnerKind::TypeConstruction,
        "crate::wire::Ticket",
    );
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
    assert_eq!(operations[0].identity.canonical, ["wire::Ticket"]);
}

#[test]
fn rooted_functional_update_does_not_borrow_local_fields() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"mod wire { pub struct Ticket { pub local_secret: u64, pub id: u64 } }
fn update(previous: ::wire::Ticket) -> ::wire::Ticket {
    ::wire::Ticket { id: 1, ..previous }
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(!index.files[0].operations.iter().any(|operation| {
        operation.kind == crate::source::SourceOperationKind::FieldRead
            && operation.identity.name == "wire::Ticket::local_secret"
    }));
    assert!(index.files[0].operations.iter().any(|operation| {
        operation.kind == crate::source::SourceOperationKind::FieldRead
            && operation.identity.name == "wire::Ticket::*"
            && operation.identity.quality == AnalysisQuality::Unresolved
    }));
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
