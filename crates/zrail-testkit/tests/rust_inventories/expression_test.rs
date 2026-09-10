//! Written expression quantities retain references, spelling, source identity, and lock binding.

use super::support::{configured_subject, coverage, exact, violation};

const SUBJECT: &str = "kind='written-expression-paths',suffixes=['State::poll','State::wake']";
const SOURCE: &str = "//! State.\npub struct State;\nimpl State { pub fn run() { State::poll(); } pub fn poll() {} pub fn wake() {} }\n";

#[test]
fn calls_and_function_values_share_expression_quantities_without_resolving_aliases() {
    let repository = configured_subject(SUBJECT, &exact("src/child.rs", "State::poll", 1), SOURCE);
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    for replacement in [
        "let _f = State::poll;",
        "crate::child::State::poll();",
        "let _f = crate::child::State::poll;",
    ] {
        repository.write(
            "src/child.rs",
            &SOURCE.replace("State::poll();", replacement),
        );
        let observed = coverage(&repository);
        assert!(observed.rust_inventories[0].satisfied);
        assert_eq!(observed.rust_inventories[0].observed_count, 1);
        assert_eq!(
            observed.rust_inventories[0].claim,
            "authored-rust-expression-path-syntax"
        );
    }
    for replacement in [
        "",
        "State::poll(); State::poll();",
        "State::wake();",
        "Self::poll();",
        "use State as Alias; Alias::poll();",
        "use State as Alias; let _f = Alias::poll;",
    ] {
        repository.write(
            "src/child.rs",
            &SOURCE.replace("State::poll();", replacement),
        );
        violation(&repository);
        let checked = repository.check();
        let finding = checked
            .report
            .findings
            .iter()
            .find(|finding| finding.id == "RUST-INVENTORY-001")
            .expect("intended expression inventory diagnostic");
        assert!(
            finding
                .message
                .contains("authored expression-path occurrences")
        );
        assert!(!finding.message.contains("method-call"));
    }
}

#[test]
fn authored_expression_worlds_include_nested_code_and_exclude_macro_tokens() {
    let code = "#[cfg(any())] State::poll(); #[cfg(test)] State::poll(); #[cfg(not(test))] State::poll(); let _f = || State::poll(); { State::poll(); }";
    let source = SOURCE.replace("State::poll();", code);
    let repository = configured_subject(SUBJECT, &exact("src/child.rs", "State::poll", 5), &source);
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    repository.write(
        "src/child.rs",
        &SOURCE.replace(
            "State::poll();",
            "stringify!(State::poll()); let _text = \"State::poll()\"; // State::poll();\n",
        ),
    );
    violation(&repository);
}

#[test]
fn physical_expression_inputs_bind_coverage_explain_and_lock_with_no_mount_duplication() {
    let repository = configured_subject(
        SUBJECT,
        &exact("src/shared.rs", "State::poll", 1),
        "//! Mounts.\npub struct State;\nmod a { include!(\"shared.rs\"); } mod b { include!(\"shared.rs\"); }\n",
    );
    repository.write("src/shared.rs", SOURCE);
    let before = coverage(&repository);
    assert!(before.rust_inventories[0].satisfied);
    assert_eq!(before.rust_inventories[0].observed_count, 1);
    assert_eq!(
        before.json().expect("JSON"),
        coverage(&repository).json().expect("repeat")
    );
    assert_eq!(
        before.rust_inventories,
        repository.explain("src/shared.rs").rust_inventories
    );
    let lock = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).expect("complete lock");
    repository.write("src/shared.rs", &format!("{SOURCE}\n// Bound input.\n"));
    let changed = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
        .expect("complete changed lock");
    assert_ne!(lock.analysis, changed.analysis);
    repository.write("src/extra.rs", SOURCE);
    violation(&repository);
}
