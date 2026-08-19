//! One-pass source facts catch aliases, hygiene, and inline tests.

use crate::inventory::{FileClass, RustSourceFile};
use zrail_core::AnalysisQuality;

use super::{SourceSyntax, index_expression, index_file};

#[test]
fn parsed_facts_preserve_architectural_evidence() {
    let source = r#"//! contract
use std::net::TcpStream as Hidden;
#[allow(dead_code)]
fn f() { let _ = Hidden::connect("localhost").unwrap(); }
#[cfg(test)] mod tests { #[test] fn proof() {} }
"#;
    let file = RustSourceFile {
        relative: "crates/a/src/worker.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse source");
    let facts = index_file(&file, &syntax);

    assert!(
        facts
            .paths
            .iter()
            .any(|path| path.name.starts_with("std::net"))
    );
    assert!(facts.methods.iter().any(|method| method.name == "unwrap"));
    assert!(!facts.lint_suppressions.is_empty());
    assert!(
        facts
            .tests
            .iter()
            .any(|test| test.name.contains("inline module"))
    );
}

#[test]
fn type_alias_function_references_remain_visible_to_call_owners() {
    let source = concat!(
        "//! type alias reference\n",
        "type Process = std::process::Command;\n",
        "fn constructor() { let _ = Process::new; }\n",
    );
    let file = RustSourceFile {
        relative: "crates/a/src/process.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse source");

    let facts = index_file(&file, &syntax);

    assert!(facts.paths.iter().any(|path| {
        path.name == "std::process::Command::new" && path.quality == AnalysisQuality::Conservative
    }));
}

#[test]
fn inline_modules_are_implementation_inside_facades() {
    let source = "//! facade\nmod hidden { pub fn work() {} }\n";
    let file = RustSourceFile {
        relative: "crates/a/src/lib.rs".into(),
        class: FileClass::Facade,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse facade");

    let facts = index_file(&file, &syntax);

    assert_eq!(facts.facade_implementation.len(), 1);
}

#[test]
fn unsafe_traits_extern_blocks_and_mutable_statics_are_visible() {
    let source = concat!(
        "//! unsafe shapes\n",
        "unsafe trait Raw {}\n",
        "unsafe extern \"C\" { fn raw(); }\n",
        "extern \"C\" { fn legacy(); }\n",
        "static mut STATE: u8 = 0;\n",
    );
    let file = RustSourceFile {
        relative: "crates/a/src/raw.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse unsafe shapes");

    let facts = index_file(&file, &syntax);
    let names = facts
        .unsafe_constructs
        .iter()
        .map(|fact| fact.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"unsafe trait"));
    assert!(names.contains(&"unsafe extern block"));
    assert!(names.contains(&"extern block"));
    assert!(names.contains(&"mutable static"));
}

#[test]
fn unsafe_attributes_in_legacy_and_2024_forms_are_visible() {
    let source = concat!(
        "//! unsafe attributes\n",
        "#[no_mangle] pub extern \"C\" fn legacy() {}\n",
        "#[export_name = \"named\"] pub fn named() {}\n",
        "#[link_section = \".raw\"] pub static RAW: u8 = 0;\n",
        "#[unsafe(no_mangle)] pub extern \"C\" fn modern() {}\n",
        "#[cfg_attr(any(), unsafe(naked))] pub fn conditional() {}\n",
    );
    let file = RustSourceFile {
        relative: "crates/a/src/attributes.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse unsafe attributes");

    let facts = index_file(&file, &syntax);
    let names = facts
        .unsafe_constructs
        .iter()
        .map(|fact| fact.name.as_str())
        .collect::<Vec<_>>();

    for name in ["export_name", "link_section", "naked", "no_mangle"] {
        assert!(
            names
                .iter()
                .any(|observed| *observed == format!("unsafe attribute {name}")),
            "missing {name}: {names:?}"
        );
    }
}

#[test]
fn expression_fragments_retain_reusable_facts() {
    let source = r#"dangerous::call(include!("nested.rs"))"#;
    let file = RustSourceFile {
        relative: "crates/a/src/value.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: 1,
    };
    let expression = syn::parse_str(source).expect("parse expression");

    let facts = index_expression(&file, &expression);

    assert_eq!(facts.syntax, SourceSyntax::Expression);
    assert!(
        facts
            .paths
            .iter()
            .any(|path| path.name == "dangerous::call")
    );
    assert_eq!(facts.includes.len(), 1);
}

#[test]
fn item_macro_invocations_are_expansion_boundaries() {
    let source = "//! macro boundary\nmacro_rules! declare { () => {} }\ndeclare!();\n";
    let file = RustSourceFile {
        relative: "crates/a/src/worker.rs".into(),
        class: FileClass::Implementation,
        source: source.into(),
        lines: source.lines().count(),
    };
    let syntax = syn::parse_file(source).expect("parse source");

    let facts = index_file(&file, &syntax);

    assert_eq!(facts.item_macros.len(), 1);
    assert_eq!(facts.item_macros[0].name, "declare");
}
