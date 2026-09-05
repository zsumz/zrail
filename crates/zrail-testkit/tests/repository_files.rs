//! Provisional file schemas cannot silently become trusted, unenforced authority.

#[path = "strict_facades/fixture.rs"]
mod fixture;

#[test]
fn unfinished_file_predicates_fail_closed_in_checks_coverage_and_lock_builds() {
    let repository = fixture::Repository::new("declarative", fixture::TEST_FACADE);
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    assert_eq!(repository.explain("src/child.rs").design_target, Some(240));
    let previous = std::fs::read(repository.0.join("zrail.lock")).expect("original authority");
    let source =
        std::fs::read_to_string(repository.0.join("zrail.toml")).expect("original contract");
    repository.write("zrail.toml", &format!("{source}\n{FILE_RULE}"));
    let config = std::path::Path::new("zrail.toml");
    let errors = [
        zrail_rust::build_lock(&repository.0, config).expect_err("no partial lock"),
        zrail_rust::governed_surface_report(&repository.0, config)
            .expect_err("no partial coverage"),
        zrail_rust::check_repository(&repository.0, config, "zrail.lock".as_ref())
            .expect_err("no ignored predicate"),
    ];
    for error in errors {
        assert!(error.to_string().contains("REP-FILE-000"), "{error}");
    }
    assert_eq!(
        std::fs::read(repository.0.join("zrail.lock")).expect("unchanged authority"),
        previous
    );
}

const FILE_RULE: &str = r#"
[[repository.files]]
name = "required-license"
include = ["LICENSE"]
reason = "The file evaluator must run before this requirement can be trusted."
predicate = { kind = "count", minimum = 1, maximum = 1 }
"#;
