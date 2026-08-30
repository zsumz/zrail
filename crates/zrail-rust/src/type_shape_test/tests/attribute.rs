//! Namespace-clean attributes do not acquire declaration-shape authority.

use super::{
    type_policy_test::{error_count, report, reset, write},
    type_shape_domain_test::{SHAPE, fixture, worlds},
};

#[test]
fn namespace_clean_attribute_still_invalidates_exact_shape() {
    let root = attributed(SHAPE, "#[reviewed::pass]\nstruct Permit { epoch: u64 }");
    let report = report(&root);
    assert_eq!(error_count(&report, "RUST-TYPE-002"), 1, "{report}");
    assert!(
        report.contains("namespace authority is not shape authority"),
        "{report}"
    );
    let coverage = super::governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    assert!(!coverage.type_policies[0].observations[0].allowed);
    assert_eq!(
        coverage.type_policies[0].observations[0].quality,
        zrail_core::AnalysisQuality::Unresolved
    );
    reset(&root);
}

#[test]
fn inactive_attribute_is_absent_but_active_and_possible_attributes_fail_closed() {
    for (enabled, expected) in [(false, 0), (true, 1)] {
        let root = attributed(
            &format!("{}\n{SHAPE}", worlds(&[enabled])),
            "#[cfg_attr(feature = \"extra\", reviewed::pass)]\nstruct Permit { epoch: u64 }",
        );
        let report = report(&root);
        assert_eq!(error_count(&report, "RUST-TYPE-002"), expected, "{report}");
        reset(&root);
    }
    let root = attributed(
        SHAPE,
        "#[cfg_attr(unix, reviewed::pass)]\nstruct Permit { epoch: u64 }",
    );
    let report = report(&root);
    assert!(
        error_count(&report, "RUST-TYPE-001") + error_count(&report, "RUST-TYPE-002") > 0,
        "{report}"
    );
    reset(&root);
}

#[test]
fn enclosing_inline_module_replacement_is_retained_on_the_declaration() {
    let root = attributed(
        &SHAPE.replace("crate::Permit", "crate::inner::Permit"),
        "#[reviewed::pass]\nmod inner { struct Permit { epoch: u64 } }",
    );
    let report = report(&root);
    assert!(
        error_count(&report, "RUST-TYPE-001") + error_count(&report, "RUST-TYPE-002") > 0,
        "{report}"
    );
    assert!(report.contains("item-replacing attribute"), "{report}");
    reset(&root);
}

#[test]
fn replacing_an_external_module_cannot_restore_exact_child_type_shape() {
    let root = attributed(
        &SHAPE
            .replace("crate::Permit", "crate::inner::Permit")
            .replace("src/lib.rs", "src/inner.rs"),
        "#[reviewed::pass]\nmod inner;",
    );
    write(&root, "src/inner.rs", "struct Permit { epoch: u64 }");
    let report = report(&root);
    assert!(
        error_count(&report, "RUST-TYPE-001") + error_count(&report, "RUST-TYPE-002") > 0,
        "{report}"
    );
    reset(&root);
}

#[test]
fn external_mount_opacity_follows_domains_and_transitive_includes() {
    for enabled in [false, true] {
        let policy = format!("{}\n{SHAPE}", worlds(&[enabled]))
            .replace("crate::Permit", "crate::inner::Permit")
            .replace("src/lib.rs", "src/fields.rs");
        let root = attributed(
            &policy,
            "#[cfg_attr(feature = \"extra\", reviewed::pass)]\nmod inner;",
        );
        write(&root, "src/inner.rs", "include!(\"fields.rs\");");
        write(&root, "src/fields.rs", "struct Permit { epoch: u64 }");
        let report = report(&root);
        assert_eq!(
            error_count(&report, "RUST-TYPE-002"),
            usize::from(enabled),
            "{report}"
        );
        reset(&root);
    }
}

fn attributed(policy: &str, source: &str) -> std::path::PathBuf {
    let policy = policy.lines().map(|line| {
        if line.starts_with("features = ") {
            format!("{line}\n[[source.rust.feature_worlds.packages]]\npackage = \"reviewed\"\ndefault_features = false\nfeatures = []")
        } else { line.into() }
    }).collect::<Vec<_>>().join("\n");
    let root = fixture(&format!("{policy}\n{ALLOWANCE}"), source);
    std::fs::create_dir_all(root.join("macros/src")).unwrap();
    write(
        &root,
        "Cargo.toml",
        concat!(
            "[workspace]\nmembers = [\"macros\"]\nresolver = \"3\"\n",
            "[package]\nname = \"policy-app\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[features]\nextra = []\n[dependencies]\nreviewed = { path = \"macros\" }\n"
        ),
    );
    write(
        &root,
        "macros/Cargo.toml",
        concat!(
            "[package]\nname = \"reviewed\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            "[lib]\nproc-macro = true\n"
        ),
    );
    write(
        &root,
        "macros/src/lib.rs",
        concat!(
            "#[proc_macro_attribute]\n",
            "pub fn pass(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream { item }\n"
        ),
    );
    root
}

const ALLOWANCE: &str = r#"[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "reviewed::pass"
namespace_effect = "none"
reason = "The reviewed attribute preserves only namespace bindings, not representation."
[source.rust.macros.allow.source]
kind = "repository"
ambient_inputs = "none"
package = "reviewed"
directory = "macros"
"#;
