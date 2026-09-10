//! Written path membership has exact physical owners without fabricated semantic identities.

use super::support::{configured_subject, coverage, incomplete, violation};

const SUBJECT: &str = "kind='written-paths-containing',names=['State']";
const OWNER: &str = "kind='exact-owners',owners=[{path='src/child.rs',name='State'}]";
const SOURCE: &str =
    "//! State.\npub struct State;\nimpl State { pub fn new() -> Self { Self } }\n";

#[test]
fn path_owners_bind_exact_authored_spelling_and_all_occurrence_evidence() {
    let repository = configured_subject(SUBJECT, OWNER, SOURCE);
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    let before = coverage(&repository);
    assert_eq!(
        before.rust_inventories[0].claim,
        "authored-rust-path-membership-syntax"
    );
    assert_eq!(before.rust_inventories[0].observed_count, 1);
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        before.rust_inventories
    );
    repository.write("src/child.rs", "//! State.\npub struct State;\n");
    violation(&repository);
    let checked = repository.check();
    assert!(
        checked
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "RUST-INVENTORY-001"
                && finding.message.contains("authored path/subject"))
    );
    repository.write(
        "src/child.rs",
        &SOURCE.replace("impl State", "impl r#State"),
    );
    violation(&repository);
    repository.write(
        "src/child.rs",
        &format!("{SOURCE}\nfn extra(x: State) {{}}\n"),
    );
    let after = coverage(&repository);
    assert!(after.rust_inventories[0].satisfied);
    assert_eq!(after.rust_inventories[0].observed_count, 2);
    assert_ne!(
        after.rust_inventories[0].inputs,
        before.rust_inventories[0].inputs
    );
}

#[test]
fn each_path_and_selected_name_counts_once_across_repeated_mounts() {
    let repository = configured_subject(
        "kind='written-paths-containing',names=['State','Unit']",
        "kind='exact-counts',counts=[{path='src/shared.rs',name='State',count=1},{path='src/shared.rs',name='Unit',count=1}]",
        "//! Mounts.\npub struct State;\nmod a { include!(\"shared.rs\"); } mod b { include!(\"shared.rs\"); }\n",
    );
    repository.write(
        "src/shared.rs",
        "//! Selected paths.\nfn run() { State::State::Unit::run(); }\n",
    );
    let report = coverage(&repository);
    let inventory = &report.rust_inventories[0];
    assert!(inventory.satisfied);
    assert_eq!(inventory.observed_count, 2);
    assert_eq!(
        inventory.occurrence_sample[0].span,
        inventory.occurrence_sample[1].span
    );
    assert_eq!(
        report.json().expect("JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
}

#[test]
fn selected_path_owners_cannot_be_hidden_by_exclusions_or_incomplete_parses() {
    let repository = configured_subject(SUBJECT, OWNER, SOURCE);
    repository.write("src/child.rs", "fn broken(\n");
    incomplete(&repository);
    repository.write("src/child.rs", SOURCE);
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write(
        "zrail.toml",
        &policy.replace("roots =", "exclude = ['src/child.rs']\nroots ="),
    );
    incomplete(&repository);
}
