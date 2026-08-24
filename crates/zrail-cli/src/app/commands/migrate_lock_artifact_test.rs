//! Migration artifact rendering is deterministic and escapes envelope strings.

use zrail_core::{
    LockMigrationClassification, LockMigrationEntry, LockMigrationReport, LockMigrationSummary,
};

use super::render;

#[test]
fn report_json_is_embedded_under_an_escaped_review_envelope() {
    let report = LockMigrationReport {
        schema: 1,
        from_semantics: 2,
        to_semantics: 3,
        summary: LockMigrationSummary {
            newly_observable: 1,
            ..LockMigrationSummary::default()
        },
        entries: vec![LockMigrationEntry {
            classification: LockMigrationClassification::NewlyObservable,
            rail: "owner".into(),
            subject: "path".into(),
            before: None,
            after: Some("exact".into()),
        }],
    };

    let rendered = render("base\"id", "contract", "report", &report).expect("render artifact");

    assert!(rendered.starts_with("{\n  \"schema\": 1,"));
    assert!(rendered.contains("\"base_commit\": \"base\\\"id\""));
    assert!(rendered.contains("\"report\": {\n    \"schema\": 1"));
    assert!(rendered.contains("\"from_semantics\": 2"));
    assert!(rendered.contains("\"classification\": \"newly-observable\""));
    assert!(rendered.ends_with("\n}\n"));
    assert_eq!(
        rendered,
        render("base\"id", "contract", "report", &report).expect("repeat render")
    );
}
