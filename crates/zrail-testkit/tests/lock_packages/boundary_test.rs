//! Missing, malformed, excessive, and undisplayed lock evidence cannot produce false passes.

use super::support::{configured, count, coverage, exact, graph, node, violation};

#[test]
fn omitted_identity_samples_never_change_counts_or_exact_set_results() {
    let nodes = (0..21)
        .map(|patch| node("wire", &format!("1.0.{patch}"), 'a'))
        .collect::<Vec<_>>();
    let repository = configured(&exact("wire", "wire", &["1.0.0"]), &graph(&nodes));
    violation(&repository, "wire");
    let policy = coverage(&repository).lock_packages.remove(0);
    assert_eq!(policy.observed_count, 21);
    assert_eq!(policy.observed_sample.len(), 16);
    assert_eq!(policy.observed_omitted, 5);
    let delta = policy.exact_difference.expect("exact differences");
    assert_eq!(delta.unexpected_count, 20);
    assert_eq!(delta.unexpected_sample.len(), 16);
    assert_eq!(delta.unexpected_omitted, 4);
    let oversized = node("wire", "1.0.1", 'a').replace("index.crates.io", &"x".repeat(20_000));
    repository.write("Cargo.lock", &graph(&[oversized]));
    violation(&repository, "wire");
    let policy = coverage(&repository).lock_packages.remove(0);
    assert_eq!(policy.observed_count, 1);
    assert!(policy.observed_sample.is_empty());
    assert_eq!(policy.observed_omitted, 1);
    let delta = policy.exact_difference.expect("oversized difference");
    assert_eq!(delta.unexpected_count, 1);
    assert_eq!(delta.unexpected_omitted, 1);
}

#[test]
fn relevant_and_unselected_lock_bytes_remain_bound_to_reviewed_analysis() {
    let repository = configured(
        &count("wire", "wire", 1),
        &graph(&[node("wire", "1.0.0", 'a')]),
    );
    repository.lock();
    let before = repository
        .check()
        .candidate_lock
        .expect("complete baseline");
    repository.write(
        "Cargo.lock",
        &format!(
            "{}\n# changed reviewed input\n",
            graph(&[node("wire", "1.0.0", 'a')])
        ),
    );
    let changed = repository.check();
    assert!(coverage(&repository).lock_packages[0].satisfied);
    assert!(
        changed
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-039")
    );
    assert_ne!(
        before.analysis,
        changed
            .candidate_lock
            .expect("complete changed input")
            .analysis
    );
    repository.write(
        "Cargo.lock",
        &graph(&[node("wire", "1.0.0", 'a'), node("unselected", "9.0.0", 'b')]),
    );
    assert!(coverage(&repository).lock_packages[0].satisfied);
    assert!(
        repository
            .check()
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-039")
    );
}

#[test]
fn missing_malformed_and_duplicate_lock_nodes_fail_before_candidate_authority() {
    let original = node("wire", "1.0.0", 'a');
    let repository = configured(
        &count("wire", "wire", 1),
        &graph(std::slice::from_ref(&original)),
    );
    for (invalid, message) in [
        (
            graph(&[original.clone(), original.clone()]),
            "duplicate package",
        ),
        (
            graph(&[original.replace("name = 'wire'", "other = 'wire'")]),
            "requires string name",
        ),
        (
            graph(&[format!("{original}\ndependencies = ['missing']")]),
            "matches no package",
        ),
        (
            graph(&[original.replace("source =", "source = false\nother =")]),
            "source must be a string",
        ),
    ] {
        repository.write("Cargo.lock", &invalid);
        let error = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
            .expect_err("no candidate for incomplete Cargo graph");
        assert!(error.to_string().contains(message), "{error}");
        assert!(zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref()).is_err());
    }
    std::fs::remove_file(repository.0.join("Cargo.lock")).expect("remove required lock input");
    let error = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
        .expect_err("missing whole-lock input");
    assert!(error.to_string().contains("require Cargo.lock"), "{error}");
}

#[test]
fn repeated_selectors_have_a_bounded_comparison_budget() {
    let huge = node("wire", "1.0.0", 'a').replace("index.crates.io", &"x".repeat(1_024 * 1_024));
    let rules = (0..65)
        .map(|index| count(&format!("wire-{index}"), "wire", 1))
        .collect::<Vec<_>>()
        .join("\n");
    let repository = configured(&rules, &graph(&[huge]));
    let error = zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref())
        .expect_err("bounded repeated graph comparison");
    assert!(
        error.to_string().contains("comparison safety limit"),
        "{error}"
    );
}
