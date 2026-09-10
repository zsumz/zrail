//! Written trait/type identities retain exact ownership and physical impl quantities.

use super::support::{configured_subject, coverage, incomplete, violation};

const SUBJECT: &str = "kind='written-trait-impls',names=['Source','SlotTransport'],implementing_types={Source=['State']}";
const DECLARATIONS: &str =
    "//! Types.\npub struct State; pub struct Other; trait Source {} trait SlotTransport {}\n";
const IMPL: &str = "impl Source for State {}";

#[test]
fn required_impl_pair_rejects_same_quantity_substitution_omission_and_growth() {
    let source = format!("{DECLARATIONS}{IMPL}\n");
    let repository = configured_subject(
        SUBJECT,
        "kind='exact-counts',counts=[{path='src/child.rs',name='Source for State',count=1}]",
        &source,
    );
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    let report = coverage(&repository);
    let observed = &report.rust_inventories[0];
    assert_eq!(observed.claim, "authored-rust-path-type-trait-impl-syntax");
    assert_eq!(observed.counts[0].name, "Source for State");
    let span = observed.occurrence_sample[0].span;
    let line = source.lines().nth(span.line - 1).expect("impl line");
    assert_eq!(&line[span.column - 1..span.end_column - 1], IMPL);
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        report.rust_inventories
    );
    for replacement in [
        "",
        "impl Source for Other {}",
        "impl SlotTransport for State {}",
        "// impl Source for State {}",
        "impl Source for State {} impl Source for State {}",
    ] {
        repository.write("src/child.rs", &format!("{DECLARATIONS}{replacement}\n"));
        violation(&repository);
    }
}

#[test]
fn type_filters_do_not_inherit_authority_from_unselected_or_unresolved_names() {
    let repository = configured_subject(SUBJECT, "kind='count',maximum=0", DECLARATIONS);
    repository.write(
        "src/child.rs",
        &format!("{DECLARATIONS}impl Source for Other {{}}\n"),
    );
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    repository.write(
        "src/child.rs",
        &format!("{DECLARATIONS}impl SlotTransport for Other {{}}\n"),
    );
    violation(&repository);
    repository.write(
        "src/child.rs",
        &format!("{DECLARATIONS}impl unknown::Source for unknown::State {{}}\n"),
    );
    violation(&repository);
    repository.write("src/child.rs", "impl Source for {");
    incomplete(&repository);
}

#[test]
fn authored_impl_sets_preserve_cfg_occurrences_and_deduplicate_repeated_mounts() {
    let repository = configured_subject(
        SUBJECT,
        "kind='exact-owners',owners=[{path='src/shared.rs',name='Source for State'}]",
        "//! Mounts.\npub struct State;\nmod a { include!(\"shared.rs\"); } mod b { include!(\"shared.rs\"); }\n",
    );
    repository.write("src/shared.rs", &format!(
        "//! Implementations.\ntrait Source {{}} struct State;\n#[cfg(test)] {IMPL}\n#[cfg(not(test))] {IMPL}\n#[cfg(any())] impl !Source for State {{}}\n"
    ));
    let report = coverage(&repository);
    assert!(report.rust_inventories[0].satisfied);
    assert_eq!(report.rust_inventories[0].observed_count, 3);
    assert_eq!(report.rust_inventories[0].counts.len(), 1);
    assert_eq!(
        report.json().expect("JSON"),
        coverage(&repository).json().expect("repeat JSON")
    );
}
