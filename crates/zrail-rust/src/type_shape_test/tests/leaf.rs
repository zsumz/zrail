//! Leaf modules include parent and sibling fragments without merging occurrences.

use super::{
    governed_surface_report,
    type_policy_test::{error_count, report, reset, write},
    type_shape_domain_test::{SHAPE, fixture},
};

#[test]
fn an_included_child_module_makes_the_containing_module_non_leaf() {
    let root = fixture(
        &policy("src/lib.rs", "crate::Permit"),
        "struct Permit { epoch: u64 }\ninclude!(\"child.rs\");",
    );
    write(&root, "src/child.rs", "mod child {}");
    non_leaf(&root);
    reset(&root);
}

#[test]
fn an_included_type_sees_child_modules_in_its_parent_and_sibling_fragments() {
    for source in [
        "mod child {}\ninclude!(\"permit.rs\");",
        "include!(\"child.rs\");\ninclude!(\"permit.rs\");",
    ] {
        let root = fixture(&policy("src/permit.rs", "crate::Permit"), source);
        write(&root, "src/permit.rs", "struct Permit { epoch: u64 }");
        write(&root, "src/child.rs", "mod child {}");
        non_leaf(&root);
        reset(&root);
    }
}

#[test]
fn repeated_mounts_retain_separate_leafness_and_occurrence_diagnostics() {
    let policies = format!(
        "{}\n{}",
        policy("src/permit.rs", "crate::leaf::Permit"),
        policy("src/permit.rs", "crate::branch::Permit").replace("permit-shape", "branch-shape")
    );
    let root = fixture(
        &policies,
        "mod leaf { include!(\"permit.rs\"); }\nmod branch { include!(\"permit.rs\"); mod child {} }",
    );
    write(&root, "src/permit.rs", "struct Permit { epoch: u64 }");
    let findings = report(&root);
    assert_eq!(error_count(&findings, "RUST-TYPE-002"), 1, "{findings}");
    assert!(findings.contains("source occurrence Some("), "{findings}");
    let coverage = governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    for entry in coverage.type_policies {
        let expected = entry.identity.contains("leaf::");
        assert!(
            entry
                .observations
                .iter()
                .all(|item| item.leaf_module == Some(expected)),
            "{entry:?}"
        );
    }
    reset(&root);
}

#[test]
fn possible_included_child_module_is_unresolved_not_a_leaf() {
    let root = fixture(
        &policy("src/lib.rs", "crate::Permit"),
        "struct Permit { epoch: u64 }\ninclude!(\"child.rs\");",
    );
    write(&root, "src/child.rs", "#[cfg(unix)] mod child {}");
    let findings = report(&root);
    assert!(
        findings.contains("child-module availability is unresolved"),
        "{findings}"
    );
    let coverage = governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
    assert!(
        coverage.type_policies[0]
            .observations
            .iter()
            .all(|item| item.leaf_module.is_none() && !item.allowed)
    );
    reset(&root);
}

#[test]
fn duplication_clean_item_macros_require_separate_exact_namespace_authority() {
    for clean_namespace in [false, true] {
        let allowance = format!(
            r#"[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "reviewed"
definition = "src/lib.rs"
duplication_effect = "none"
namespace_effect = {:?}
reason = "The exact reviewed expansion has independently reviewed effects."
[[source.rust.item_macros]]
name = "reviewed"
path = "src/expansion.rs"
binding = "exact"
reason = "This expansion introduces no source-file mounts."
"#,
            if clean_namespace { "none" } else { "opaque" }
        );
        let root = fixture(
            &format!("{}\n{allowance}", policy("src/permit.rs", "crate::Permit")),
            "macro_rules! reviewed { () => {} }\ninclude!(\"permit.rs\");\ninclude!(\"expansion.rs\");",
        );
        write(&root, "src/permit.rs", "struct Permit { epoch: u64 }");
        write(&root, "src/expansion.rs", "reviewed!();");
        let findings = report(&root);
        assert_eq!(
            error_count(&findings, "RUST-TYPE-002") == 0,
            clean_namespace,
            "{findings}"
        );
        if !clean_namespace {
            assert!(
                findings.contains("logical module namespace is opaque"),
                "{findings}"
            );
        }
        let coverage = governed_surface_report(&root, "zrail.toml".as_ref()).unwrap();
        let declarations = coverage.type_policies[0]
            .observations
            .iter()
            .filter(|item| item.operation == "declaration")
            .collect::<Vec<_>>();
        assert!(!declarations.is_empty());
        assert!(
            declarations
                .iter()
                .all(|item| item.allowed == clean_namespace),
            "{declarations:?}"
        );
        assert!(
            declarations
                .iter()
                .all(|item| item.leaf_module == clean_namespace.then_some(true))
        );
        reset(&root);
    }
}

fn policy(path: &str, identity: &str) -> String {
    SHAPE
        .replace("src/lib.rs", path)
        .replace("crate::Permit", identity)
        .replace(
            "visibility = \"private\"\nreason",
            "visibility = \"private\"\nleaf_module = true\nreason",
        )
}

fn non_leaf(root: &std::path::Path) {
    let findings = report(root);
    assert_eq!(error_count(&findings, "RUST-TYPE-002"), 1, "{findings}");
    let coverage = governed_surface_report(root, "zrail.toml".as_ref()).unwrap();
    assert!(
        coverage.type_policies[0]
            .observations
            .iter()
            .all(|item| item.leaf_module == Some(false) && !item.allowed)
    );
}
