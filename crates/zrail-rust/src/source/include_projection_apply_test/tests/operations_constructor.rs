//! Constructor ownership follows the value namespace without capitalization guesses.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::*;
use crate::source::{SourceIndex, SourceOperationKind};

#[test]
fn lowercase_cross_module_tuple_constructor_is_exact() {
    assert_constructor(
        "#[allow(non_camel_case_types)] pub struct ticket(pub u64);",
        "fn mint() { let _ = super::model::ticket(44); }",
        "crate::model::ticket",
    );
}

#[test]
fn lowercase_tuple_constructor_import_alias_is_exact() {
    assert_constructor(
        "#[allow(non_camel_case_types)] pub struct ticket(pub u64);",
        "use super::model::ticket as make; fn mint() { let _ = make(44); }",
        "crate::model::ticket",
    );
}

#[test]
fn lowercase_unit_constructor_import_alias_is_exact() {
    assert_constructor(
        "#[allow(non_camel_case_types)] pub struct marker;",
        "use super::model::marker as value; fn mint() { let _ = value; }",
        "crate::model::marker",
    );
}

#[test]
fn lowercase_enum_variant_import_alias_is_exact() {
    assert_constructor(
        "#[allow(non_camel_case_types)] pub enum choice { ready(u64) }",
        "use super::model::choice::ready as make; fn mint() { let _ = make(44); }",
        "crate::model::choice::ready",
    );
}

#[test]
fn glob_imported_lowercase_constructor_reaches_owner() {
    let (mut index, domain, modules) = fixture(
        "#[allow(non_camel_case_types)] pub enum choice { ready(u64) }",
        "use super::model::choice::*; fn mint() { let _ = ready(44); }",
    );
    let findings = canonicalize_operations(&mut index, &domain, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = matching_operations(
        &index,
        "src/user.rs",
        OwnerKind::TypeConstruction,
        "crate::model::choice::ready",
    );
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(
        operations[0].identity.quality,
        AnalysisQuality::Conservative
    );
}

#[test]
fn raw_identifier_constructor_alias_is_exact() {
    assert_constructor(
        "#[allow(non_camel_case_types)] pub struct ticket(pub u64);",
        "use super::model::ticket as r#match; fn mint() { let _ = r#match(44); }",
        "crate::model::ticket",
    );
}

#[test]
fn uppercase_cross_module_tuple_constructor_becomes_exact() {
    assert_constructor(
        "pub struct Ticket(pub u64);",
        "fn mint() { let _ = super::model::Ticket(44); }",
        "crate::model::Ticket",
    );
}

#[test]
fn proven_values_and_wrong_constructor_forms_are_discarded() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        r"#[allow(non_snake_case)] fn Ticket(_: u64) {}
#[allow(non_upper_case_globals)] const Marker: u64 = 1;
struct Tuple(u64);
struct Unit;
struct Choice;
impl Choice { fn ticket(_: u64) {} }
fn run(make: fn(u64)) {
    Ticket(1);
    let _ = Marker;
    make(2);
    let local = 3;
    let _ = local;
    let _ = Tuple;
    let _ = Unit();
    Choice::ticket(4);
}",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(
        index.files[0]
            .operations
            .iter()
            .all(|operation| operation.kind != SourceOperationKind::TypeConstruction),
        "non-constructors survived: {:#?}",
        index.files[0].operations,
    );
}

fn assert_constructor(model: &str, user: &str, selector: &str) {
    let (mut index, domain, modules) = fixture(model, user);
    let findings = canonicalize_operations(&mut index, &domain, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations =
        matching_operations(&index, "src/user.rs", OwnerKind::TypeConstruction, selector);
    assert_eq!(operations.len(), 1, "owner {selector}: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Exact);
}

fn fixture(
    model: &str,
    user: &str,
) -> (
    SourceIndex,
    crate::source::CompilationDomain,
    Vec<crate::source::CompilationModuleEdge>,
) {
    let root = parsed_file("src/lib.rs", "mod model; mod user;");
    let model = parsed_file("src/model.rs", model);
    let user = parsed_file("src/user.rs", user);
    let domain = domain();
    let modules = [("model", "src/model.rs"), ("user", "src/user.rs")]
        .into_iter()
        .map(|(name, child)| {
            module_edge(
                "src/lib.rs",
                name,
                child,
                module(&root.modules, name),
                &domain,
            )
        })
        .collect();
    (index([root, model, user]), domain, modules)
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
