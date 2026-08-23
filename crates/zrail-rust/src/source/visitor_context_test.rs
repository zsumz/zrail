//! Include boundaries inherit cfg(test) from every enclosing syntax shape.

use std::collections::BTreeMap;

use syn::visit::Visit;

use super::super::{imports::ImportMap, visitor::FactVisitor};
use crate::source::SyntaxGuard;

#[test]
fn enclosing_items_expressions_and_locals_are_test_only() {
    let source = r#"
        struct Harness;

        #[cfg(test)]
        fn function() { include!("function.rs"); }

        #[cfg(test)]
        const FIXTURE: () = { include!("const.rs") };

        #[cfg(test)]
        impl Harness {
            fn inherited() { include!("impl.rs"); }
        }

        impl Harness {
            #[cfg(test)]
            fn method() { include!("method.rs"); }
        }

        fn local() {
            #[cfg(test)]
            let _ = include!("local.rs");
            let _ = match () {
                #[cfg(test)]
                () => include!("arm.rs"),
                _ => (),
            };
        }
    "#;

    let includes = includes(source);

    for path in [
        "function.rs",
        "const.rs",
        "impl.rs",
        "method.rs",
        "local.rs",
        "arm.rs",
    ] {
        assert_eq!(includes.get(path), Some(&true), "{path}");
    }
}

#[test]
fn file_inner_cfg_applies_to_all_boundaries() {
    let includes = includes("#![cfg(test)]\ninclude!(\"file.rs\");\n");

    assert_eq!(includes.get("file.rs"), Some(&true));
}

#[test]
fn ordinary_include_remains_production_context() {
    let includes = includes("fn production() { include!(\"production.rs\"); }\n");

    assert_eq!(includes.get("production.rs"), Some(&false));
}

#[test]
fn observed_facts_retain_test_only_syntax_guards() {
    let syntax = syn::parse_file(
        r#"
        pub fn production() { std::fs::read("input"); }
        #[cfg(test)]
        mod tests {
            fn guarded() {
                std::process::Command::new("true");
                let _ = include_str!("fixture.txt");
            }
        }
        "#,
    )
    .expect("parse guarded facts");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    let filesystem = visitor
        .paths
        .iter()
        .find(|fact| fact.name == "std::fs::read")
        .expect("production fact");
    let process = visitor
        .paths
        .iter()
        .find(|fact| fact.name == "std::process::Command::new")
        .expect("guarded fact");
    let compile = visitor
        .compile_effects
        .iter()
        .find(|fact| fact.target.as_deref() == Some("fixture.txt"))
        .expect("guarded compile effect");

    assert_eq!(filesystem.guard, SyntaxGuard::Ordinary);
    assert_eq!(process.guard, SyntaxGuard::TestOnly);
    assert_eq!(compile.invocation.guard, SyntaxGuard::TestOnly);
}

#[test]
fn statement_macro_inputs_retain_outer_test_only_guard() {
    let syntax = syn::parse_file(
        r#"
        pub fn run() {
            #[cfg(test)]
            assert!(std::process::Command::new("true").status().is_ok());
        }
        "#,
    )
    .expect("parse guarded statement macro");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    let path = visitor
        .paths
        .iter()
        .find(|fact| fact.name == "std::process::Command::new")
        .expect("inspected process path");
    let call = visitor
        .calls
        .iter()
        .find(|fact| fact.name == "std::process::Command::new")
        .expect("inspected process call");
    let expansion = visitor
        .macro_expansions
        .iter()
        .find(|expansion| expansion.name == "assert")
        .expect("assert expansion");

    assert_eq!(path.guard, SyntaxGuard::TestOnly);
    assert_eq!(call.guard, SyntaxGuard::TestOnly);
    assert_eq!(expansion.guard, SyntaxGuard::TestOnly);
}

fn includes(source: &str) -> BTreeMap<String, bool> {
    let syntax = syn::parse_file(source).expect("parse cfg context fixture");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);
    visitor
        .includes
        .into_iter()
        .map(|boundary| {
            (
                boundary.path.expect("literal include boundary"),
                boundary.cfg_test,
            )
        })
        .collect()
}
