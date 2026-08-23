//! Block modules and crate aliases use module, not block, namespace ancestry.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn inline_block_module_super_qualifiers_ignore_the_block_alias() {
    let root = fixture("namespace-block-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn host() { use std::fs::File as Spawn; mod child { pub fn direct() { let _ = super::Spawn::new(\"sh\"); } pub fn nested() { let _ = self::super::Spawn::new(\"sh\"); } } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_calls(&check(&root), "src/lib.rs", 2);
    reset(&root);
}

#[test]
fn included_block_module_super_qualifiers_ignore_the_block_alias() {
    let root = fixture("namespace-included-block-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nuse std::process::Command as Spawn;\npub fn host() { use std::fs::File as Spawn; let _ = include!(\"block.rs\"); }\n",
    );
    write(
        &root,
        "src/block.rs",
        "{ mod child { pub fn direct() { let _ = super::Spawn::new(\"sh\"); } pub fn nested() { let _ = self::super::Spawn::new(\"sh\"); } } 0 }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_calls(&check(&root), "src/block.rs", 2);
    reset(&root);
}

#[test]
fn extern_crate_self_alias_survives_nested_include_and_local_shadow() {
    let root = fixture("namespace-extern-self", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nextern crate self as root;\nmod executor;\npub mod bridge { pub use std::process::Command as Spawn; }\nmod shadow {}\nmod outer { use crate::shadow as root; include!(\"root_call.rs\"); }\n",
    );
    write(
        &root,
        "src/root_call.rs",
        "mod nested { pub fn hidden() { let _ = ::root::bridge::Spawn::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_exact_owned_calls(&check(&root), "src/root_call.rs", 1);
    reset(&root);
}

fn assert_exact_owned_calls(report: &Report, path: &str, expected: usize) {
    let findings = report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "process-spawn"
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Exact
        })
        .count();
    assert_eq!(findings, expected, "{}", report.human());
}
