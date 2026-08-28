//! One physical include fragment can have multiple Rust syntax identities.

use super::fixture::{Repository, assert_complete, call_owner, exact_owner_count, finding_count};

#[test]
fn same_fragment_supports_top_level_and_impl_item_syntax() {
    let repository = Repository::new(
        "shared-items-and-impl-items",
        "mod owner; include!(\"shared.inc\"); pub struct Process; impl Process { include!(\"shared.inc\"); }",
        "",
        "",
    );
    repository.write("src/shared.inc", "fn helper() {}");

    let report = repository.check();
    assert_source_complete(&report);
    assert_eq!(report.analysis.rust_files, 3, "{}", report.human());
}

#[test]
fn module_items_survive_a_second_impl_item_occurrence() {
    let repository = Repository::new(
        "shared-module-and-impl-items",
        "mod owner; mod shared; pub struct Process; impl Process { include!(\"shared.rs\"); }",
        "",
        "",
    );
    repository.write("src/shared.rs", "fn helper() {}");

    let report = repository.check();
    assert_source_complete(&report);
    assert_eq!(report.analysis.rust_files, 3, "{}", report.human());
}

#[test]
fn same_fragment_supports_impl_and_trait_item_contexts() {
    let contract = format!(
        "{}\n{}",
        call_owner("process-launch", "crate::Process::launch"),
        call_owner("trait-launch", "crate::Launch::launch"),
    );
    let repository = Repository::new(
        "shared-impl-and-trait-items",
        "mod owner; pub trait Launch { fn launch(); include!(\"shared.inc\"); } pub struct Process; impl Process { pub fn launch() {} include!(\"shared.inc\"); }",
        "pub fn own_process() { crate::Process::launch(); } pub fn own_trait<T: crate::Launch>() { T::launch(); }",
        &contract,
    );
    repository.write("src/shared.inc", "fn relay() { Self::launch(); }");

    let report = repository.check();
    assert_source_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "process-launch", "src/shared.inc"),
        1,
        "{}",
        report.human()
    );
    assert_eq!(
        finding_count(
            &report,
            "RUST-CALL-001",
            "rust.source.call-resolution",
            "src/shared.inc",
        ),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn non_rs_top_level_item_fragment_is_analyzed() {
    let repository = Repository::new(
        "non-rs-top-level-items",
        "mod owner; pub struct Process; impl Process { pub fn launch() {} } include!(\"items.fragment\");",
        "pub fn own() { crate::Process::launch(); }",
        &call_owner("process-launch", "crate::Process::launch"),
    );
    repository.write(
        "src/items.fragment",
        "pub fn trespass() { Process::launch(); }",
    );

    let report = repository.check();
    assert_source_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "process-launch", "src/items.fragment"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn syntax_variants_count_as_one_physical_file() {
    let repository = Repository::new(
        "physical-fragment-metric",
        "mod owner; include!(\"shared.inc\"); pub struct Process; impl Process { include!(\"shared.inc\"); }",
        "",
        "",
    );
    repository.write("src/shared.inc", "fn helper() {}");

    let report = repository.check();
    assert_source_complete(&report);
    assert_eq!(report.analysis.rust_files, 3, "{}", report.human());
}

fn assert_source_complete(report: &zrail_core::Report) {
    assert_complete(report);
    assert!(report.analysis.complete, "{}", report.human());
    assert!(
        report.findings.iter().all(|finding| {
            !finding.id.starts_with("RUST-PARSE-") && !finding.id.starts_with("RUST-GRAPH-")
        }),
        "{}",
        report.human()
    );
}
