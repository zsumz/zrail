//! Associated values terminate constructor candidates at their canonical self type.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::super::{
    canonicalize_operations, canonicalize_operations_with_external, domain, matching_operations,
    module, module_edge, parsed_file,
};
use crate::source::{CompilationModuleEdge, SourceIndex};

#[test]
fn cross_module_inherent_method_is_proven_value() {
    assert_no_construction("impl crate::model::State { pub fn version() -> u64 { 1 } }");
}

#[test]
fn cross_module_associated_const_is_proven_value() {
    assert_no_construction("impl crate::model::State { pub const FACTORY: fn() -> u64 = || 1; }");
}

#[test]
fn cross_module_trait_method_is_proven_value() {
    assert_no_construction(
        "trait Versioned { fn version() -> u64; } impl Versioned for crate::model::State { fn version() -> u64 { 1 } }",
    );
}

#[test]
fn glob_imported_trait_method_is_proven_value() {
    assert_no_construction(
        "mod traits { pub trait Versioned { fn version() -> u64; } } use traits::*; impl Versioned for crate::model::State { fn version() -> u64 { 1 } }",
    );
}

#[test]
fn cfg_inactive_associated_item_does_not_suppress_unknown_candidate() {
    let mut index = fixture(
        "#[cfg(feature = \"disabled\")] impl crate::model::State { pub fn version() -> u64 { 1 } }",
        "let _ = super::model::State::version();",
    );
    let mut compilation = domain();
    compilation.feature_world = Some("default".into());
    let modules = modules_for_domain(&index, &compilation);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    let operations = owned(&index);
    assert_eq!(operations.len(), 1, "operations: {operations:#?}");
    assert_eq!(operations[0].identity.quality, AnalysisQuality::Unresolved);
}

#[test]
fn same_named_item_on_another_type_does_not_suppress_candidate() {
    let mut index = fixture(
        "struct Other; impl Other { pub fn version() -> u64 { 1 } }",
        "let _ = super::model::State::version();",
    );
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(owned(&index).len(), 1);
}

#[test]
fn local_trait_impl_for_external_self_is_proven_value() {
    let mut index = trait_fixture(
        "pub trait Versioned { fn version() -> u64; }",
        "impl crate::traits::Versioned for external::State { fn version() -> u64 { 1 } }",
        "use crate::traits::Versioned; let _ = external::State::version();",
    );
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings =
        canonicalize_operations_with_external(&mut index, &compilation, &modules, "external");

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(
        owned_for(&index, "external::State").is_empty(),
        "operations: {:#?}",
        owned_for(&index, "external::State")
    );
}

#[test]
fn inherent_external_impl_does_not_suppress_candidate() {
    assert_external_candidate(
        "",
        "impl external::State { fn version() -> u64 { 1 } }",
        "let _ = external::State::version();",
    );
}

#[test]
fn external_trait_for_external_self_does_not_suppress_candidate() {
    assert_external_candidate(
        "",
        "impl external::Versioned for external::State { fn version() -> u64 { 1 } }",
        "let _ = external::State::version();",
    );
}

#[test]
fn cross_module_default_trait_method_is_proven_value() {
    assert_trait_default(
        "pub trait Versioned { fn version() -> u64 { 1 } }",
        "let _ = crate::model::State::version();",
    );
}

#[test]
fn cross_module_default_trait_const_is_proven_value() {
    assert_trait_default(
        "pub trait Versioned { const VERSION: u64 = 1; }",
        "let _ = crate::model::State::VERSION;",
    );
}

#[test]
fn required_trait_item_without_definition_does_not_suppress_candidate() {
    let mut index = trait_fixture(
        "pub trait Versioned { fn version() -> u64; }",
        "impl crate::traits::Versioned for crate::model::State {}",
        "use crate::traits::Versioned; let _ = crate::model::State::version();",
    );
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(owned(&index).len(), 1);
}

fn assert_no_construction(implementation: &str) {
    let call = if implementation.contains("FACTORY") {
        "let _ = super::model::State::FACTORY();"
    } else {
        "let _ = super::model::State::version();"
    };
    let mut index = fixture(implementation, call);
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(owned(&index).is_empty(), "operations: {:#?}", owned(&index));
}

fn assert_external_candidate(trait_declaration: &str, implementation: &str, call: &str) {
    let mut index = trait_fixture(trait_declaration, implementation, call);
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings =
        canonicalize_operations_with_external(&mut index, &compilation, &modules, "external");

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert_eq!(owned_for(&index, "external::State").len(), 1);
}

fn assert_trait_default(trait_declaration: &str, call: &str) {
    let mut index = trait_fixture(
        trait_declaration,
        "impl crate::traits::Versioned for crate::model::State {}",
        &format!("use crate::traits::Versioned; {call}"),
    );
    let compilation = domain();
    let modules = modules_for_domain(&index, &compilation);
    let findings = canonicalize_operations(&mut index, &compilation, &modules);

    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    assert!(owned(&index).is_empty(), "operations: {:#?}", owned(&index));
}

fn owned(index: &SourceIndex) -> Vec<crate::source::SourceOperationFact> {
    owned_for(index, "crate::model::State")
}

fn owned_for(index: &SourceIndex, selector: &str) -> Vec<crate::source::SourceOperationFact> {
    matching_operations(index, "src/user.rs", OwnerKind::TypeConstruction, selector)
}

fn fixture(implementation: &str, call: &str) -> SourceIndex {
    trait_fixture("", implementation, call)
}

fn trait_fixture(trait_declaration: &str, implementation: &str, call: &str) -> SourceIndex {
    let root = parsed_file(
        "src/lib.rs",
        "mod model; mod traits; mod extensions; mod user;",
    );
    let model = parsed_file("src/model.rs", "pub struct State;");
    let traits = parsed_file("src/traits.rs", trait_declaration);
    let extensions = parsed_file("src/extensions.rs", implementation);
    let user = parsed_file("src/user.rs", &format!("fn inspect() {{ {call} }}"));
    SourceIndex {
        files: vec![root, model, traits, extensions, user],
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}

fn modules_for_domain(
    index: &SourceIndex,
    compilation: &crate::source::CompilationDomain,
) -> Vec<CompilationModuleEdge> {
    let root = index
        .files
        .iter()
        .find(|file| file.relative == "src/lib.rs")
        .expect("root file");
    [
        ("model", "src/model.rs"),
        ("traits", "src/traits.rs"),
        ("extensions", "src/extensions.rs"),
        ("user", "src/user.rs"),
    ]
    .into_iter()
    .map(|(name, child)| {
        module_edge(
            "src/lib.rs",
            name,
            child,
            module(&root.modules, name),
            compilation,
        )
    })
    .collect()
}
