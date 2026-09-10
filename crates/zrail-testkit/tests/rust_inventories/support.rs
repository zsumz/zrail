//! Valid offline repositories isolate inventory assertions from unrelated failures.

use super::fixture::{Repository, TEST_FACADE};

pub(super) const SOURCE: &str = "//! State.\npub struct State;\nimpl State { pub fn run(&self) { self.poll(); } fn poll(&self) {} fn wake(&self) {} }\n";

pub(super) fn configured(assertion: &str, source: &str) -> Repository {
    configured_subject(
        "kind='written-methods',names=['poll','wake']",
        assertion,
        source,
    )
}

pub(super) fn configured_subject(subject: &str, assertion: &str, source: &str) -> Repository {
    let repository = Repository::new("declarative", TEST_FACADE);
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("contract");
    repository.write("zrail.toml", &format!(
        "{policy}\n[[source.rust.inventories]]\nname='methods'\nreason='Reviewed written transport syntax.'\ninclude=['src/**/*.rs']\nworld='authored'\nsubject={{{subject}}}\nassertion={{{assertion}}}\n"
    ));
    repository.write("src/child.rs", source);
    repository.write(
        "Cargo.lock",
        "version=4\n[[package]]\nname='fixture'\nversion='0.0.0'\n",
    );
    repository
}

pub(super) fn exact(path: &str, name: &str, count: usize) -> String {
    format!("kind='exact-counts',counts=[{{path='{path}',name='{name}',count={count}}}]")
}

pub(super) fn coverage(repository: &Repository) -> zrail_rust::GovernedSurfaceReport {
    zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("complete Rust inventory")
}

pub(super) fn violation(repository: &Repository) {
    let checked = repository.check();
    assert!(checked.analysis.is_complete());
    assert!(
        checked.report.findings.iter().any(|finding| {
            finding.id == "RUST-INVENTORY-001" && finding.rule == "rust:inventory:methods"
        }),
        "{:?}",
        checked.report
    );
    assert!(!coverage(repository).rust_inventories[0].satisfied);
}

pub(super) fn incomplete(repository: &Repository) {
    let error =
        zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).expect_err("no partial lock");
    assert!(error.to_string().contains("RUST-INVENTORY-002"), "{error}");
    let error = zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect_err("no partial coverage");
    assert!(error.to_string().contains("RUST-INVENTORY-002"), "{error}");
}
