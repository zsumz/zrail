//! Prelude bindings retain canonical traits and enum constructors.

use zrail_core::{AnalysisQuality, OwnerKind};

use super::super::{
    canonicalize_operations, domain, matching_operations, module, module_edge, parsed_file,
};
use crate::source::{CallResolutionKind, SourceIndex};

#[path = "prelude/shadows.rs"]
mod shadows;

#[test]
fn prelude_default_trait_is_exact() {
    assert_exact_trait("fn make<T: Default>() -> T { <T as Default>::default() }");
}

#[test]
fn prelude_into_trait_is_exact() {
    assert_exact_trait("fn convert<T: Into<U>, U>(value: T) -> U { <T as Into<U>>::into(value) }");
}

#[test]
fn prelude_iterator_trait_is_exact() {
    assert_exact_trait(
        "fn step<I: Iterator>(iter: &mut I) -> Option<I::Item> { <I as Iterator>::next(iter) }",
    );
}

#[test]
fn no_std_uses_core_prelude() {
    let index =
        canonicalized("#![no_std]\nfn make<T: Default>() -> T { <T as Default>::default() }");

    let boundaries = explicit_trait_boundaries(&index);
    assert!(boundaries.is_empty(), "boundaries: {boundaries:#?}");
}

#[test]
fn std_prelude_injects_to_owned_trait() {
    let index = canonicalized(
        "fn clone<T: ToOwned>(value: &T) { let _ = <T as ToOwned>::to_owned(value); }",
    );

    assert_no_explicit_trait_boundaries(&index);
}

#[test]
fn rust_2018_does_not_inject_2021_prelude_traits() {
    let mut edition = domain();
    edition.edition = "2018".into();
    let index = canonicalized_in(
        "fn convert<T: TryInto<U>, U>(value: T) -> U { <T as TryInto<U>>::try_into(value).ok().unwrap() }",
        &edition,
    );

    assert_eq!(explicit_trait_boundaries(&index).len(), 1);
}

#[test]
fn rust_2021_injects_2021_prelude_traits() {
    let mut edition = domain();
    edition.edition = "2021".into();
    let index = canonicalized_in(
        "fn convert<T: TryInto<U>, U>(value: T) { let _ = <T as TryInto<U>>::try_into(value); }",
        &edition,
    );

    assert_no_explicit_trait_boundaries(&index);
}

#[test]
fn rust_2021_does_not_inject_2024_prelude_traits() {
    let mut edition = domain();
    edition.edition = "2021".into();
    let index = canonicalized_in(
        "fn poll<T: Future>() { let _ = <T as Future>::poll; }",
        &edition,
    );

    assert_eq!(explicit_trait_boundaries(&index).len(), 1);
}

#[test]
fn rust_2024_injects_2024_prelude_traits() {
    let index = canonicalized("fn poll<T: Future>() { let _ = <T as Future>::poll; }");

    assert_no_explicit_trait_boundaries(&index);
}

#[test]
fn no_implicit_prelude_disables_implicit_trait() {
    let root = parsed_file("src/lib.rs", "#[no_implicit_prelude] mod disabled;");
    let disabled = parsed_file("src/disabled.rs", "mod nested;");
    let nested = parsed_file(
        "src/disabled/nested.rs",
        "fn make<T>() -> T { <T as Default>::default() }",
    );
    let domain = domain();
    let modules = vec![
        module_edge(
            "src/lib.rs",
            "disabled",
            "src/disabled.rs",
            module(&root.modules, "disabled"),
            &domain,
        ),
        module_edge(
            "src/disabled.rs",
            "nested",
            "src/disabled/nested.rs",
            module(&disabled.modules, "nested"),
            &domain,
        ),
    ];
    let mut index = index([root, disabled, nested]);
    let findings = canonicalize_operations(&mut index, &domain, &modules);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
    assert_eq!(explicit_trait_boundaries(&index).len(), 1);
    assert!(!has_path(&index, "core::default::Default"));
}

#[test]
fn local_trait_shadows_prelude_trait() {
    assert_shadowing_trait(
        "trait Default { fn default() -> Self; } fn make<T: Default>() -> T { <T as Default>::default() }",
        "Default::default",
    );
}

#[test]
fn explicit_import_shadows_prelude_trait() {
    assert_shadowing_trait(
        "mod custom { pub trait Default { fn default() -> Self; } } use custom::Default; fn make<T: Default>() -> T { <T as Default>::default() }",
        "custom::Default::default",
    );
}

#[test]
fn self_qualified_name_does_not_consult_prelude() {
    let mut index = index([parsed_file(
        "src/lib.rs",
        "fn make<T>() -> T { <T as self::Default>::default() }",
    )]);
    let findings = canonicalize_operations(&mut index, &domain(), &[]);

    assert!(
        findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002")
    );
    assert_eq!(explicit_trait_boundaries(&index).len(), 1);
    assert!(!has_path(&index, "core::default::Default"));
}

#[test]
fn import_target_does_not_consult_prelude() {
    let index =
        canonicalized("use Default as Alias; fn make<T>() -> T { <T as Alias>::default() }");

    assert_eq!(explicit_trait_boundaries(&index).len(), 1);
    assert!(!has_path(&index, "core::default::Default"));
}

#[test]
fn prelude_some_reaches_option_variant_owner() {
    assert_owned_constructor(
        "fn run() { let _ = Some(1_u8); }",
        "core::option::Option::Some",
        1,
    );
}

#[test]
fn prelude_none_reaches_option_variant_owner() {
    assert_owned_constructor(
        "fn run() { let _: Option<u8> = None; }",
        "core::option::Option::None",
        1,
    );
}

#[test]
fn prelude_ok_and_err_reach_result_variant_owners() {
    let index = canonicalized(
        "fn run() { let _: Result<u8, u8> = Ok(1); let _: Result<u8, u8> = Err(2); }",
    );

    assert_owned(&index, "core::result::Result::Ok", 1);
    assert_owned(&index, "core::result::Result::Err", 1);
}

fn assert_exact_trait(source: &str) {
    let index = canonicalized(source);
    assert_no_explicit_trait_boundaries(&index);
}

fn assert_shadowing_trait(source: &str, selected: &str) {
    let index = canonicalized(source);
    assert_no_explicit_trait_boundaries(&index);
    assert_path(&index, selected);
    assert!(!has_path(&index, "core::default::Default"));
    assert!(!has_path(&index, "core::default::Default::default"));
}

fn assert_owned_constructor(source: &str, selector: &str, count: usize) {
    let index = canonicalized(source);
    assert_owned(&index, selector, count);
}

fn assert_owned(index: &SourceIndex, selector: &str, count: usize) {
    let operations =
        matching_operations(index, "src/lib.rs", OwnerKind::TypeConstruction, selector);
    assert_eq!(operations.len(), count, "operations: {operations:#?}");
    assert!(
        operations
            .iter()
            .all(|operation| operation.identity.quality == AnalysisQuality::Exact)
    );
}

fn assert_no_explicit_trait_boundaries(index: &SourceIndex) {
    assert!(
        explicit_trait_boundaries(index).is_empty(),
        "boundaries: {:#?}",
        explicit_trait_boundaries(index)
    );
}

fn explicit_trait_boundaries(index: &SourceIndex) -> Vec<&crate::source::CallResolutionFact> {
    index
        .files
        .iter()
        .flat_map(|file| &file.call_resolutions)
        .filter(|fact| fact.kind == CallResolutionKind::ExplicitTrait)
        .collect()
}

fn assert_path(index: &SourceIndex, name: &str) {
    assert!(has_path(index, name), "missing {name}: {:#?}", index.files);
}

fn has_path(index: &SourceIndex, name: &str) -> bool {
    index
        .files
        .iter()
        .flat_map(|file| &file.paths)
        .any(|fact| fact.name == name && fact.quality == AnalysisQuality::Exact)
}

fn has_call(index: &SourceIndex, name: &str) -> bool {
    index
        .files
        .iter()
        .flat_map(|file| &file.calls)
        .any(|fact| fact.name == name && fact.quality == AnalysisQuality::Exact)
}

fn canonicalized(source: &str) -> SourceIndex {
    canonicalized_in(source, &domain())
}

fn canonicalized_in(source: &str, compilation: &crate::source::CompilationDomain) -> SourceIndex {
    let mut index = index([parsed_file("src/lib.rs", source)]);
    let findings = canonicalize_operations(&mut index, compilation, &[]);
    assert!(findings.is_empty(), "unexpected findings: {findings:#?}");
    index
}

fn index(files: impl IntoIterator<Item = crate::source::RustFileFacts>) -> SourceIndex {
    SourceIndex {
        files: files.into_iter().collect(),
        findings: Vec::new(),
        analysis_metrics: crate::source::SourceAnalysisMetrics::default(),
    }
}
