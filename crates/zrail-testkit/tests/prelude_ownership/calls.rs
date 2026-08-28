//! Call, capability, scope, directive, and lexical-shadow prelude behavior.

use super::fixture::{Repository, assert_no_owner, count, exact, findings};

#[test]
fn bare_drop_reaches_canonical_call_owner() {
    let repository = fixture("bare-drop", "pub fn trespass() { drop(1_u8); }");
    exact(&repository.check(), "OWN-003", "drop-call", "src/lib.rs");
}

#[test]
fn bare_drop_reference_reaches_capability_owner() {
    let repository = fixture(
        "drop-reference",
        "pub fn trespass() { let _disposer: fn(u8) = drop; }",
    );
    exact(
        &repository.check(),
        "OWN-003",
        "drop-capability",
        "src/lib.rs",
    );
}

#[test]
fn vec_new_reaches_canonical_call_owner() {
    let repository = fixture("vec-new", "pub fn trespass() { let _ = Vec::<u8>::new(); }");
    exact(&repository.check(), "OWN-003", "vec-new", "src/lib.rs");
}

#[test]
fn prelude_type_reaches_denied_symbol_scope() {
    let repository = fixture(
        "vec-scope",
        "pub fn trespass() { let _: Vec<u8> = Vec::new(); }",
    );
    exact(&repository.check(), "CAP-001", "vec-symbols", "src/lib.rs");
}

#[test]
fn local_drop_shadows_prelude_call() {
    let repository = fixture(
        "local-drop",
        "use core::mem::drop;\npub fn local(drop: fn(u8)) { drop(1); }",
    );
    let report = repository.check();
    let evidence = findings(&report, "OWN-003", "drop-call", "src/lib.rs");
    assert_eq!(evidence.len(), 1, "{}", report.human());
    assert!(evidence[0].span.is_none(), "{}", report.human());
}

#[test]
fn no_implicit_prelude_does_not_canonicalize_drop() {
    let repository = Repository::new(
        "no-implicit",
        "2024",
        "#![no_implicit_prelude]\nmod owner; pub fn local() { drop(1_u8); }",
    );
    let report = repository.check();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
    assert_no_owner(&report, "drop-call", "src/lib.rs");
}

#[test]
fn no_std_does_not_canonicalize_vec() {
    let repository = Repository::new(
        "no-std-vec",
        "2024",
        "#![no_std]\nextern crate std; mod owner; pub fn local() { let _ = Vec::<u8>::new(); }",
    );
    let report = repository.check();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
    assert_no_owner(&report, "vec-new", "src/lib.rs");
    assert_eq!(count(&report, "CAP-001", "vec-symbols", "src/lib.rs"), 0);
}

#[test]
fn mixed_cache_modes_preserve_before_and_after_let() {
    let repository = fixture(
        "mixed-shadow-cache",
        "use core::mem::drop; pub fn mixed() { drop(1_u8); let drop = |_: u8| {}; drop(2); }",
    );
    let report = repository.check();
    assert_eq!(count(&report, "OWN-003", "drop-call", "src/lib.rs"), 1);
}

#[test]
fn type_generic_drop_does_not_shadow_value_prelude() {
    let repository = fixture(
        "type-generic-drop",
        "#[allow(non_camel_case_types)] pub fn trespass<drop>() { drop(1_u8); }",
    );
    exact(&repository.check(), "OWN-003", "drop-call", "src/lib.rs");
}

#[test]
fn expression_include_inherits_drop_shadow() {
    let repository = fixture(
        "included-shadow",
        "pub fn local(drop: fn(u8)) { include!(\"expr.rs\"); }",
    );
    repository.write("src/expr.rs", "{ drop(1); }");
    assert_no_owner(&repository.check(), "drop-call", "src/expr.rs");
}

#[test]
fn nested_expression_include_inherits_drop_shadow() {
    let repository = fixture(
        "nested-included-shadow",
        "pub fn local(drop: fn(u8)) { include!(\"outer.rs\"); }",
    );
    repository.write("src/outer.rs", "{ include!(\"inner.rs\") }");
    repository.write("src/inner.rs", "{ drop(1); }");
    assert_no_owner(&repository.check(), "drop-call", "src/inner.rs");
}

#[test]
fn conditional_expression_include_shadow_fails_closed() {
    let repository = fixture(
        "conditional-included-shadow",
        "pub fn local() { #[cfg(unix)] let drop = |_: u8| {}; include!(\"expr.rs\"); }",
    );
    repository.write("src/expr.rs", "{ drop(1_u8); }");
    let report = repository.check();
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}

#[test]
fn expression_include_inherits_const_generic_shadow() {
    let repository = fixture(
        "const-generic-included-shadow",
        "#[allow(non_upper_case_globals)] pub fn local<const drop: usize>() { include!(\"expr.rs\"); }",
    );
    repository.write("src/expr.rs", "{ let _ = drop; }");
    assert_no_owner(&repository.check(), "drop-capability", "src/expr.rs");
}

fn fixture(name: &str, body: &str) -> Repository {
    Repository::new(name, "2024", &format!("mod owner; {body}"))
}
