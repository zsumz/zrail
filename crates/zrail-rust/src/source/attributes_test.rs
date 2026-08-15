//! Attribute recognition examples.

use syn::ItemFn;

use super::{
    is_cfg_test, is_lint_suppression, is_test_attribute, lint_suppression_is_reasoned,
    path_attribute,
};

#[test]
fn cfg_attr_is_not_a_test_detector_escape() {
    let function =
        syn::parse_str::<ItemFn>("#[cfg_attr(any(), test)] fn proof() {}").expect("parse function");
    assert!(function.attrs.iter().any(is_test_attribute));
}

#[test]
fn cfg_test_helper_is_not_a_test_body() {
    let function = syn::parse_str::<ItemFn>("#[cfg(test)] fn helper() {}").expect("parse function");

    assert!(!function.attrs.iter().any(is_test_attribute));
}

#[test]
fn conditional_lint_suppressions_are_visible() {
    let function = syn::parse_str::<ItemFn>("#[cfg_attr(any(), allow(dead_code))] fn f() {}")
        .expect("parse function");
    assert!(function.attrs.iter().any(is_lint_suppression));
}

#[test]
fn lint_suppressions_require_a_nonempty_reason() {
    let reasoned = syn::parse_str::<ItemFn>(
        "#[allow(clippy::too_many_arguments, reason = \"protocol boundary\")] fn f() {}",
    )
    .expect("parse function");
    let empty = syn::parse_str::<ItemFn>(
        r#"#[expect(clippy::too_many_arguments, reason = "  ")] fn f() {}"#,
    )
    .expect("parse function");
    let missing =
        syn::parse_str::<ItemFn>("#[allow(dead_code)] fn f() {}").expect("parse function");

    assert!(reasoned.attrs.iter().any(lint_suppression_is_reasoned));
    assert!(!empty.attrs.iter().any(lint_suppression_is_reasoned));
    assert!(!missing.attrs.iter().any(lint_suppression_is_reasoned));
}

#[test]
fn conditional_suppressions_are_reasoned_only_when_every_branch_is() {
    let mixed = syn::parse_str::<ItemFn>(
        "#[cfg_attr(any(), allow(dead_code, reason = \"generated glue\"), expect(unused))] fn f() {}",
    )
    .expect("parse function");

    assert!(mixed.attrs.iter().any(is_lint_suppression));
    assert!(!mixed.attrs.iter().any(lint_suppression_is_reasoned));
}

#[test]
fn cfg_not_test_is_not_a_sibling_test_guard() {
    let module =
        syn::parse_str::<syn::ItemMod>("#[cfg(not(test))] mod worker_test;").expect("parse module");

    assert!(!module.attrs.iter().any(is_cfg_test));
}

#[test]
fn cfg_all_test_is_a_sibling_test_guard() {
    let module =
        syn::parse_str::<syn::ItemMod>("#[cfg(all(test, feature = \"tls\"))] mod worker_test;")
            .expect("parse module");

    assert!(module.attrs.iter().any(is_cfg_test));
}

#[test]
fn cfg_any_test_is_not_a_sibling_test_guard() {
    let module = syn::parse_str::<syn::ItemMod>(
        "#[cfg(any(test, feature = \"standalone\"))] mod worker_test;",
    )
    .expect("parse module");

    assert!(!module.attrs.iter().any(is_cfg_test));
}

#[test]
fn sibling_path_attributes_are_extracted_exactly() {
    let module = syn::parse_str::<syn::ItemMod>(
        "#[cfg(test)] #[path = \"worker_test.rs\"] mod worker_test;",
    )
    .expect("parse module");

    assert_eq!(
        path_attribute(&module.attrs).as_deref(),
        Some("worker_test.rs")
    );
}
