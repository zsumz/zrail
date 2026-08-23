//! Path consumers distinguish type-only locals from same-name imported values.

use zrail_core::{AnalysisQuality, Report};

use super::super::{check, fixture, reset, write, write_executor, write_lock};

#[test]
fn expression_path_uses_the_glob_imported_value_not_the_local_type() {
    let root = fixture("namespace-type-value-expression", FILESYSTEM_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nmod fs_owner;\nstruct Spawn { _private: () }\nuse bridge::*;\npub fn hidden() { let f = Spawn; let _ = f(\"input\"); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\npub use std::fs::read as Spawn;\n",
    );
    write(
        &root,
        "src/fs_owner.rs",
        "//! Filesystem owner.\npub fn allowed() { let _ = std::fs::read(\"input\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    for (id, rule) in [
        ("OWN-003", "filesystem-capability"),
        ("CAP-001", "filesystem-symbols"),
        ("EFFECT-001", "profile.filesystem"),
    ] {
        assert_non_unresolved(&report, id, rule);
    }
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn type_position_keeps_the_local_type_not_the_glob_imported_value() {
    let root = fixture("namespace-type-value-control", TYPE_CONTROL_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\npub struct Spawn { _private: () }\nuse bridge::*;\npub fn local(value: Spawn) -> Spawn { value }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\npub use std::fs::read as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "CAP-001"
                && finding.rule == "local-type"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule == "filesystem-symbols"),
        "{}",
        report.human()
    );
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn test_only_foreign_function_does_not_shadow_a_production_glob_value() {
    let root = fixture("namespace-cfg-foreign-value", FILESYSTEM_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nmod fs_owner;\nuse bridge::*;\n#[cfg(test)]\nunsafe extern \"C\" { fn Spawn(path: *const u8); }\npub fn hidden() { let _ = Spawn(\"input\"); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\npub use std::fs::read as Spawn;\n",
    );
    write(
        &root,
        "src/fs_owner.rs",
        "//! Filesystem owner.\npub fn allowed() { let _ = std::fs::read(\"input\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    for (id, rule) in [
        ("OWN-003", "filesystem-capability"),
        ("OWN-003", "filesystem-read"),
        ("CAP-001", "filesystem-symbols"),
        ("EFFECT-001", "profile.filesystem"),
    ] {
        assert_non_unresolved(&report, id, rule);
    }
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn module_and_value_alias_with_the_same_spelling_reach_the_reexported_value() {
    let root = fixture("namespace-module-value-same-spelling", FILESYSTEM_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nmod fs_owner;\nmod foo { pub use std::fs::read as Bar; }\nuse foo::Bar as foo;\npub fn hidden() { let _ = foo(\"input\"); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Capability owner marker.\npub use std::fs::File as Marker;\n",
    );
    write(
        &root,
        "src/fs_owner.rs",
        "//! Filesystem owner.\npub fn allowed() { let _ = std::fs::read(\"input\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    for (id, rule) in [
        ("OWN-003", "filesystem-capability"),
        ("OWN-003", "filesystem-read"),
        ("CAP-001", "filesystem-symbols"),
        ("EFFECT-001", "profile.filesystem"),
    ] {
        assert_non_unresolved(&report, id, rule);
    }
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn unit_struct_value_wins_over_a_same_name_filesystem_glob() {
    let contract = format!("{FILESYSTEM_CONTRACT}{LOCAL_VALUE_CONTRACT}");
    let root = fixture("namespace-unit-value-control", &contract);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nstruct Spawn;\nuse bridge::*;\npub fn local() { let _ = Spawn; }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\npub use std::fs::read as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "CAP-001"
                && finding.rule == "local-value"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis == AnalysisQuality::Exact
        }),
        "{}",
        report.human()
    );
    for rule in [
        "filesystem-capability",
        "filesystem-read",
        "filesystem-symbols",
        "profile.filesystem",
    ] {
        assert_no_rule_in_lib(&report, rule);
    }
    assert_no_include(&report);
    reset(&root);
}

fn assert_non_unresolved(report: &Report, id: &str, rule: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == id
                && finding.rule == rule
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.analysis != AnalysisQuality::Unresolved
        }),
        "{}",
        report.human()
    );
}

fn assert_no_include(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}

fn assert_no_rule_in_lib(report: &Report, rule: &str) {
    assert!(
        !report.findings.iter().any(|finding| {
            finding.rule == rule && finding.path.as_deref() == Some("src/lib.rs")
        }),
        "{}",
        report.human()
    );
}

const FILESYSTEM_CONTRACT: &str = r#"
[profiles.filesystem.effects]
deny = ["filesystem"]

[[layer]]
name = "filesystem-app"
packages = ["fixture"]
profiles = ["filesystem"]
reason = "The caller exposes filesystem effects."

[[scope]]
name = "filesystem-symbols"
include = ["src/lib.rs"]
reason = "The caller may not reach filesystem symbols."
[scope.symbols]
deny = ["std::fs"]

[[owner]]
name = "filesystem-capability"
kind = "capability"
within = ["src/**"]
match = "std::fs"
allow = ["src/bridge.rs", "src/fs_owner.rs"]
reason = "Only the bridge and owner may name filesystem capabilities."

[[owner]]
name = "filesystem-read"
kind = "call"
within = ["src/**"]
match = "std::fs::read"
allow = ["src/fs_owner.rs"]
reason = "Only the filesystem owner may invoke reads."
"#;

const TYPE_CONTROL_CONTRACT: &str = r#"
[[scope]]
name = "local-type"
include = ["src/lib.rs"]
reason = "The type position resolves to the local type."
[scope.symbols]
deny = ["Spawn"]

[[scope]]
name = "filesystem-symbols"
include = ["src/lib.rs"]
reason = "Type positions must not borrow a same-name value import."
[scope.symbols]
deny = ["std::fs"]
"#;

const LOCAL_VALUE_CONTRACT: &str = r#"
[[scope]]
name = "local-value"
include = ["src/lib.rs"]
reason = "The value position resolves to the local unit constructor."
[scope.symbols]
deny = ["Spawn"]
"#;
