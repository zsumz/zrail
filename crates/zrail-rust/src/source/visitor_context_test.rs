//! Include boundaries inherit cfg(test) from every enclosing syntax shape.

use std::collections::BTreeMap;

use syn::visit::Visit;

use super::super::{imports::ImportMap, visitor::FactVisitor};

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
