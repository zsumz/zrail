//! Small fixtures assert intended policy identities rather than unrelated failing exits.

use super::fixture::{Repository, TEST_FACADE};

pub(super) fn configured(rules: &str) -> Repository {
    let repository = Repository::new("declarative", TEST_FACADE);
    let source = std::fs::read_to_string(repository.0.join("zrail.toml")).expect("base contract");
    repository.write("zrail.toml", &format!("{source}\n{rules}"));
    repository
}

pub(super) fn violation(repository: &Repository, policy: &str, diagnostic: &str) {
    let result = repository.check();
    assert!(
        result
            .report
            .findings
            .iter()
            .any(|finding| finding.id == diagnostic
                && finding.rule == format!("repository:file:{policy}")),
        "{policy}: {:?}",
        result.report
    );
}

pub(super) fn coverage(repository: &Repository) -> zrail_rust::GovernedSurfaceReport {
    zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
        .expect("complete repository-file coverage")
}
