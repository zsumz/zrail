//! Opaque aliases and generated glob members never receive exact call authority.

use zrail_core::{AnalysisQuality, Report};

use super::super::{
    assert_no_owner_findings, check, fixture, reset, write, write_executor, write_lock,
};

#[test]
fn type_only_local_name_does_not_suppress_a_glob_imported_value() {
    let root = fixture("namespace-type-value-glob", FILESYSTEM_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nmod fs_owner;\nstruct Spawn { _private: () }\nuse bridge::*;\npub fn hidden() { let _ = Spawn(\"input\"); }\n",
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
        ("OWN-003", "filesystem-read"),
        ("OWN-003", "filesystem-capability"),
        ("CAP-001", "filesystem-symbols"),
        ("EFFECT-001", "profile.filesystem"),
    ] {
        assert_non_unresolved(&report, id, rule, "src/lib.rs");
    }
    assert_no_include(&report);
    reset(&root);
}

#[test]
fn complete_globbed_module_with_item_include_has_no_missing_member_uncertainty() {
    let root = fixture("namespace-complete-include-glob", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod bridge;\nmod benign_source;\nuse bridge::*;\nuse benign_source::*;\npub fn clean() { benign(); }\n",
    );
    write(
        &root,
        "src/bridge.rs",
        "//! Bridge.\ninclude!(\"members.rs\");\n",
    );
    write(&root, "src/members.rs", "pub fn present() {}\n");
    write(
        &root,
        "src/benign_source.rs",
        "//! Benign source.\npub fn benign() {}\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_no_include(&report);
    assert_no_owner_findings(&report, "process-spawn");
    reset(&root);
}

#[test]
fn opaque_associated_type_alias_fails_closed() {
    let root = fixture("namespace-opaque-associated", OPAQUE_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ntrait Provider { type Command; }\nstruct Runtime;\nimpl Provider for Runtime { type Command = std::process::Command; }\ntype Spawn = <Runtime as Provider>::Command;\npub fn allowed() { let _ = Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_fail_closed(&check(&root), "opaque-member");
    reset(&root);
}

#[test]
fn macro_generated_member_reached_through_a_module_glob_fails_closed() {
    let root = fixture("namespace-macro-glob-member", GLOB_MEMBER_CONTRACT);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmacro_rules! members { () => { pub struct Spawn; impl Spawn { pub fn new(_: &str) -> Self { Self } } } }\nmod bridge { members!(); }\nuse bridge::*;\npub fn allowed() { let _ = Spawn::new(\"local\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_fail_closed(&check(&root), "glob-member");
    reset(&root);
}

fn assert_fail_closed(report: &Report, rule: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002"
                || (finding.rule == rule
                    && finding.id.starts_with("OWN-")
                    && finding.analysis != AnalysisQuality::Exact)
        }),
        "{}",
        report.human()
    );
}

fn assert_non_unresolved(report: &Report, id: &str, rule: &str, path: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == id
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
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
reason = "Only the filesystem owner may read files."
"#;

const OPAQUE_CONTRACT: &str = r#"
[[owner]]
name = "opaque-member"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Associated-type aliases cannot receive exact direct-call authority."
"#;

const GLOB_MEMBER_CONTRACT: &str = r#"
[[owner]]
name = "glob-member"
kind = "call"
within = ["src/**"]
match = "Spawn::new"
allow = ["src/lib.rs"]
reason = "Generated glob members cannot receive exact direct-call authority."
"#;
