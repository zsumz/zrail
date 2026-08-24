//! Visibility anchors never become runtime or capability path facts.

use syn::visit::Visit;

use zrail_core::AnalysisQuality;

use super::super::{imports::ImportMap, visitor::FactVisitor};

#[test]
fn restricted_visibility_paths_are_not_observed_authority() {
    let syntax = syn::parse_file(
        r#"
        pub(crate) struct Root;
        mod outer {
            pub(super) struct Parent;
            mod inner {
                pub(in crate::outer) fn run() { std::fs::read("input"); }
            }
        }
        "#,
    )
    .expect("parse visibility fixture");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    assert!(
        visitor
            .paths
            .iter()
            .any(|fact| fact.name == "std::fs::read")
    );
    assert!(
        !visitor
            .paths
            .iter()
            .any(|fact| { matches!(fact.name.as_str(), "crate" | "super" | "crate::outer") })
    );
}

#[test]
fn macro_and_attribute_names_are_not_observed_as_ordinary_paths() {
    let syntax = syn::parse_file(
        "#[test] fn example() { assert!(std::path::Path::new(\"input\").exists()); }",
    )
    .expect("parse macro fixture");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    assert!(
        visitor
            .paths
            .iter()
            .any(|fact| fact.name == "std::path::Path::new")
    );
    assert!(!visitor.paths.iter().any(|fact| fact.name == "test"));
    assert!(!visitor.paths.iter().any(|fact| fact.name == "assert"));
    assert!(visitor.opaque_binding_macros.is_empty());
}

#[test]
fn associated_callable_projection_is_not_an_exact_raw_path() {
    let syntax = syn::parse_file(
        "trait Provider { type Command; } struct Runtime; fn run() { let constructor = <Runtime as Provider>::Command::new; }",
    )
    .expect("parse associated projection");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);

    assert_eq!(visitor.call_resolutions.len(), 1);
    assert!(
        !visitor.paths.iter().any(|fact| {
            fact.name == "Provider::Command::new" && fact.quality == AnalysisQuality::Exact
        }),
        "{:#?}",
        visitor.paths
    );
}
