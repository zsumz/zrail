//! Include fact tests distinguish literal and generated source boundaries.

use super::{IncludeContext, include_boundary};
use crate::source::SyntaxGuard;

#[test]
fn literal_and_generated_includes_are_distinct() {
    let literal = parse_macro(r#"include!("local.rs");"#);
    let boundary = include_boundary(&literal, IncludeContext::Items).expect("literal include");
    assert_eq!(boundary.path.as_deref(), Some("local.rs"));
    assert_eq!(boundary.out_dir, None);
    assert!(!boundary.generated);
    assert_eq!(boundary.guard, SyntaxGuard::Ordinary);
    assert_eq!(boundary.context, IncludeContext::Items);

    let generated = parse_macro(r#"include!(concat!(env!("OUT_DIR"), "/generated.rs"));"#);
    let boundary =
        include_boundary(&generated, IncludeContext::Expression).expect("generated include");
    assert_eq!(boundary.path, None);
    assert_eq!(boundary.out_dir.as_deref(), Some("generated.rs"));
    assert!(boundary.generated);
    assert_eq!(boundary.context, IncludeContext::Expression);
}

#[test]
fn out_dir_bindings_require_one_canonical_builtin_expression() {
    for source in [
        r#"include!(concat!(env!("OTHER"), "/generated.rs"));"#,
        r#"include!(concat!(env!("OUT_DIR"), "/../generated.rs"));"#,
        r#"include!(concat!(env!("OUT_DIR"), "/nested", "/generated.rs"));"#,
    ] {
        let boundary = include_boundary(&parse_macro(source), IncludeContext::Items)
            .expect("include boundary");
        assert_eq!(boundary.out_dir, None, "{source}");
    }
}

fn parse_macro(source: &str) -> syn::Macro {
    syn::parse_str::<syn::ItemMacro>(source)
        .expect("parse macro")
        .mac
}
