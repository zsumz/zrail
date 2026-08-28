//! Associated-item includes are exact graph edges with inherited lexical context.

use super::fixture::{Repository, assert_complete, call_owner, exact_owner_count, finding_count};

#[test]
fn impl_item_include_becomes_source_graph_edge() {
    let repository = impl_repository("impl-item-edge", "include!(\"impl_items.inc\");", "");
    repository.write("src/impl_items.inc", "fn included() {}");
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(graph_findings(&report), 0, "{}", report.human());
}

#[test]
fn impl_item_include_inherits_concrete_self() {
    let repository = impl_repository(
        "impl-item-self",
        "include!(\"impl_items.inc\");",
        &call_owner("process-spawn", "crate::Process::spawn"),
    );
    repository.write("src/impl_items.inc", "fn trespass() { Self::spawn(); }");
    assert_owned_call(&repository, "src/impl_items.inc");
}

#[test]
fn trait_item_include_inherits_current_trait() {
    let repository = Repository::new(
        "trait-item-self",
        "mod owner; pub trait Launch { include!(\"trait_items.inc\"); } pub struct Process; impl Launch for Process {}",
        "pub fn own() { <crate::Process as crate::Launch>::launch(); }",
        &call_owner("launch", "crate::Launch::launch"),
    );
    repository.write("src/trait_items.inc", "fn launch() { Self::launch(); }");
    assert_eq!(
        resolution_count(&repository.check(), "src/trait_items.inc"),
        1
    );
}

#[test]
fn associated_item_include_inherits_generic_bounds() {
    let repository = Repository::new(
        "impl-item-generics",
        "mod owner; pub trait Factory { fn ready(); } pub struct Process<T>(T); impl<T: Factory> Process<T> { include!(\"impl_items.inc\"); } pub struct Worker; impl Factory for Worker { fn ready() {} }",
        "pub fn own() { <crate::Worker as crate::Factory>::ready(); }",
        &call_owner("factory-ready", "crate::Factory::ready"),
    );
    repository.write("src/impl_items.inc", "fn trespass() { T::ready(); }");
    assert_eq!(
        resolution_count(&repository.check(), "src/impl_items.inc"),
        1
    );
}

#[test]
fn non_rs_associated_fragment_is_analyzed() {
    let repository = impl_repository(
        "non-rs-associated-fragment",
        "include!(\"methods.fragment\");",
        &call_owner("process-spawn", "crate::Process::spawn"),
    );
    repository.write("src/methods.fragment", "fn trespass() { Self::spawn(); }");
    assert_owned_call(&repository, "src/methods.fragment");
}

#[test]
fn associated_fragment_findings_keep_fragment_coordinates() {
    let repository = impl_repository(
        "associated-fragment-span",
        "include!(\"methods.inc\");",
        &call_owner("process-spawn", "crate::Process::spawn"),
    );
    repository.write("src/methods.inc", "fn trespass() { Self::spawn(); }");
    let report = repository.check();
    let span = report
        .findings
        .iter()
        .find(|finding| {
            finding.id == "OWN-003"
                && finding.rule == "process-spawn"
                && finding.path.as_deref() == Some("src/methods.inc")
        })
        .and_then(|finding| finding.span);
    assert_eq!(span.map(|span| (span.line, span.column)), Some((1, 17)));
}

#[test]
fn nested_associated_item_include_is_transitive() {
    let repository = impl_repository(
        "nested-associated-fragment",
        "include!(\"outer.inc\");",
        &call_owner("process-spawn", "crate::Process::spawn"),
    );
    repository.write("src/outer.inc", "include!(\"inner.inc\");");
    repository.write("src/inner.inc", "fn trespass() { Self::spawn(); }");
    assert_owned_call(&repository, "src/inner.inc");
}

fn impl_repository(name: &str, body: &str, contract: &str) -> Repository {
    Repository::new(
        name,
        &format!("mod owner; pub struct Process; impl Process {{ pub fn spawn() {{}} {body} }}"),
        "pub fn own() { crate::Process::spawn(); }",
        contract,
    )
}

fn assert_owned_call(repository: &Repository, path: &str) {
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "process-spawn", path),
        1,
        "{}",
        report.human()
    );
}

fn resolution_count(report: &zrail_core::Report, path: &str) -> usize {
    finding_count(report, "RUST-CALL-001", "rust.source.call-resolution", path)
}

fn graph_findings(report: &zrail_core::Report) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| finding.id.starts_with("RUST-GRAPH-"))
        .count()
}
