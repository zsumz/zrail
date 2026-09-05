//! Empty, future, and differently mounted selectors cannot acquire guessed authority.

use super::{
    fixture::Repository,
    support::{SCOPE, measured},
};

#[test]
fn unknown_package_selector_fails_instead_of_disabling_the_override() {
    let repository = Repository::new("declarative", &SCOPE.replace("[\"fixture\"]", "[\"typo\"]"));
    let error =
        zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).expect_err("unknown package");
    assert!(
        error
            .to_string()
            .contains("unknown workspace package \"typo\""),
        "{error}"
    );
}

#[test]
fn future_overlap_is_rejected_by_hypothetical_explain_and_when_the_file_arrives() {
    let first = SCOPE.replace("src/**", "src/future*");
    let second = SCOPE
        .replace("src/**", "src/future.rs")
        .replace("name = \"core\"", "name = \"other\"");
    let repository = Repository::new("declarative", &format!("{first}{second}"));
    repository.lock();
    let error = zrail_rust::explain_hypothetical_path(
        &repository.0,
        "zrail.toml".as_ref(),
        "src/future.rs".as_ref(),
    )
    .expect_err("future overlap");
    assert!(error.to_string().contains("RUST-SIZE-006"), "{error}");
    repository.write("src/future.rs", &measured(3));
    repository.write("src/lib.rs", "//! Wiring.\nmod child;\nmod future;\n");
    assert!(zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).is_err());
}

#[test]
fn line_measurement_keeps_crlf_and_unterminated_final_records() {
    let repository = Repository::new("declarative", SCOPE);
    for source in [
        measured(6),
        measured(6).replace('\n', "\r\n"),
        measured(6).trim_end_matches('\n').to_owned(),
    ] {
        repository.write("src/child.rs", &source);
        let report = zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref())
            .expect("coverage");
        assert_eq!(
            report
                .size_budgets
                .iter()
                .find(|budget| budget.path == "src/child.rs")
                .expect("physical source")
                .lines,
            6
        );
    }
}
