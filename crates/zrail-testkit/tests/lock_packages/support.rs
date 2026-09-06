//! Offline fixtures carry valid complete Cargo graphs so policy failures stay attributable.

use super::fixture::{Repository, TEST_FACADE};

pub(super) fn configured(rules: &str, lock: &str) -> Repository {
    let repository = Repository::new("declarative", TEST_FACADE);
    let policy = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("base contract");
    repository.write("zrail.toml", &format!("{policy}\n{rules}"));
    repository.write("Cargo.lock", lock);
    repository
}

pub(super) fn count(name: &str, package: &str, count: usize) -> String {
    format!(
        "[[dependencies.lock_package]]\nname = '{name}'\npackage = '{package}'\nreason = 'Reviewed whole-lock quantity.'\nassertion = {{ kind = 'count', count = {count} }}\n"
    )
}

pub(super) fn exact(name: &str, package: &str, versions: &[&str]) -> String {
    let identities = versions.iter().map(|version| format!(
        "{{ version = '{version}', source = 'registry+https://index.crates.io', checksum = '{}' }}", "a".repeat(64),
    )).collect::<Vec<_>>().join(", ");
    format!(
        "[[dependencies.lock_package]]\nname = '{name}'\npackage = '{package}'\nreason = 'Reviewed whole-lock identities.'\nassertion = {{ kind = 'exact', identities = [{identities}] }}\n"
    )
}

pub(super) fn graph(nodes: &[String]) -> String {
    format!(
        "version = 4\n[[package]]\nname = 'fixture'\nversion = '0.0.0'\n{}",
        nodes.join("\n")
    )
}

pub(super) fn node(name: &str, version: &str, checksum: char) -> String {
    format!(
        "[[package]]\nname = '{name}'\nversion = '{version}'\nsource = 'registry+https://index.crates.io'\nchecksum = '{}'\n",
        checksum.to_string().repeat(64)
    )
}

pub(super) fn coverage(repository: &Repository) -> zrail_rust::GovernedSurfaceReport {
    zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("complete inventory report")
}

pub(super) fn violation(repository: &Repository, name: &str) {
    let result = repository.check();
    assert!(result.analysis.is_complete());
    assert!(
        result.report.findings.iter().any(|finding| {
            finding.id == "DEP-LOCK-001"
                && finding.rule == format!("dependency:lock-package:{name}")
                && finding.path.as_deref() == Some("Cargo.lock")
        }),
        "{name}: {:?}",
        result.report
    );
}
