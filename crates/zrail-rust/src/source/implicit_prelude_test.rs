//! Prelude directives preserve crate, module, and conditional scope.

use super::{PreludeDirectiveKind, directives};
use crate::source::SyntaxGuard;

use super::super::implicit_prelude_catalog::{PreludeItemKind, core, std_only};

#[test]
fn crate_directives_have_root_scope() {
    let syntax =
        syn::parse_file("#![no_std]\n#![no_implicit_prelude]\n").expect("parse crate directives");
    let facts = directives(&syntax);

    assert_eq!(facts.len(), 2);
    assert!(facts.iter().all(|fact| fact.lexical_scope.is_empty()));
    assert!(
        facts
            .iter()
            .any(|fact| fact.kind == PreludeDirectiveKind::NoStd)
    );
    assert!(facts.iter().any(|fact| {
        fact.kind == PreludeDirectiveKind::NoImplicit && fact.guard == SyntaxGuard::Ordinary
    }));
}

#[test]
fn module_directive_uses_the_module_scope() {
    let syntax = syn::parse_file("#[no_implicit_prelude] mod outer { mod inner { fn run() {} } }")
        .expect("parse module directive");
    let facts = directives(&syntax);

    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].kind, PreludeDirectiveKind::NoImplicit);
    assert_eq!(facts[0].lexical_scope.len(), 1);
}

#[test]
fn cfg_attr_directive_retains_its_feature_guard() {
    let syntax = syn::parse_file("#![cfg_attr(feature = \"minimal\", no_std)]\nfn run() {}")
        .expect("parse conditional directive");
    let facts = directives(&syntax);

    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].kind, PreludeDirectiveKind::NoStd);
    assert!(facts[0].guard.canonical_name().contains("minimal"));
}

#[test]
fn core_trait_identities_are_canonical() {
    for (name, canonical) in [
        ("Default", "core::default::Default"),
        ("Into", "core::convert::Into"),
        ("Iterator", "core::iter::Iterator"),
    ] {
        let entry = core(name, "2024").expect("core prelude trait");
        assert_eq!(entry.canonical, canonical);
        assert_eq!(entry.kind, PreludeItemKind::Type);
    }
}

#[test]
fn enum_constructor_identities_are_canonical() {
    for (name, canonical, kind) in [
        (
            "Some",
            "core::option::Option::Some",
            PreludeItemKind::TupleConstructor,
        ),
        (
            "None",
            "core::option::Option::None",
            PreludeItemKind::UnitConstructor,
        ),
        (
            "Ok",
            "core::result::Result::Ok",
            PreludeItemKind::TupleConstructor,
        ),
        (
            "Err",
            "core::result::Result::Err",
            PreludeItemKind::TupleConstructor,
        ),
    ] {
        let entry = core(name, "2024").expect("core prelude constructor");
        assert_eq!(entry.canonical, canonical);
        assert_eq!(entry.kind, kind);
    }
}

#[test]
fn edition_and_std_only_entries_stay_bounded() {
    assert!(core("TryInto", "2018").is_none());
    assert!(core("TryInto", "2021").is_some());
    assert!(core("Future", "2021").is_none());
    assert!(core("Future", "2024").is_some());
    assert_eq!(
        std_only("ToOwned").expect("std prelude trait").canonical,
        "std::borrow::ToOwned"
    );
}
