//! Cross-file local types and projected places retain exact field authority.

use zrail_core::AnalysisQuality;
use zrail_rust::{check_repository, governed_surface_report};

#[path = "local_field_ownership/fixture.rs"]
mod fixture;

use fixture::{repository, reset};

#[test]
fn inferred_locals_and_projected_writes_preserve_narrow_owners() {
    let root = repository();
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check local field ownership fixture")
        .report;

    for rule in [
        "commit-index-authority",
        "applied-index-authority",
        "configuration-authority",
        "leader-replacement-authority",
    ] {
        assert!(
            report.findings.iter().all(|finding| {
                finding.rule != rule || !matches!(finding.id.as_str(), "OWN-003" | "OWN-006")
            }),
            "{rule}: {}",
            report.human()
        );
    }
    let coverage = governed_surface_report(&root, "zrail.toml".as_ref())
        .expect("report exact local field ownership");
    for name in ["commit-index-authority", "applied-index-authority"] {
        let owner = coverage
            .owners
            .iter()
            .find(|owner| owner.name == name)
            .expect("index owner coverage");
        assert!(
            owner.occurrences.iter().any(|occurrence| {
                occurrence.path == "src/node/construction.rs"
                    && occurrence.quality == AnalysisQuality::Exact
            }),
            "{name}: {owner:#?}"
        );
    }
    let configuration = coverage
        .owners
        .iter()
        .find(|owner| owner.name == "configuration-authority")
        .expect("configuration owner coverage");
    assert!(
        configuration
            .occurrences
            .iter()
            .all(|occurrence| occurrence.path == "src/node/log.rs"),
        "{configuration:#?}"
    );
    let leader = coverage
        .owners
        .iter()
        .find(|owner| owner.name == "leader-replacement-authority")
        .expect("leader replacement coverage");
    assert_eq!(leader.occurrences.len(), 1, "{leader:#?}");
    assert_eq!(leader.occurrences[0].path, "src/node/lifecycle.rs");
    assert_eq!(leader.occurrences[0].operation, "field-write");
    reset(&root);
}
