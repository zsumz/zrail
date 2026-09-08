//! Physical failure paths distinguish exact parity, fail-closed differences and native-only cycles.

use super::model::{self, Fixture, Fragment, NAMES};
use std::fs;
use zrail_core::RepositoryEntryMode;

pub(super) fn policies() -> Fragment {
    let bytes = fs::read_to_string(
        model::project().join("docs/rc9/policies/kafka-driver.retired-boundary.fragment.toml"),
    )
    .expect("proposed boundary policy");
    let proposed: Fragment = toml::from_str(&bytes).expect("strict proposed policy");
    let mut expected = model::policies();
    for rule in &mut expected.repository.files {
        if rule.name.starts_with("kd-retired-tree-") {
            rule.entry = RepositoryEntryMode::NonDirectory;
        }
    }
    assert_eq!(proposed.repository.files, expected.repository.files);
    assert_eq!(
        proposed.source.rust.inventories,
        expected.source.rust.inventories
    );
    proposed
}

pub(super) fn check(fixture: &Fixture, name: &str, legacy: Option<bool>, diagnostic: &str) {
    if let Some(accepted) = legacy {
        assert_eq!(
            !super::physical::contains_rust_source(&fixture.root.join("src/reactor").join(name)),
            accepted,
            "unchanged original walker: {name}"
        );
    }
    let result = crate::repository_files::analyze(&fixture.root, &policies().repository.files);
    if diagnostic == "REP-FILE-006" {
        let error = result.expect_err("unread boundary must not certify absence");
        assert!(error.starts_with("REP-FILE-006:"), "{error}");
        assert!(
            error.contains(&format!("src/reactor/{name}")),
            "intended boundary: {error}"
        );
    } else {
        let result = result.expect("complete physical selection");
        let actual = result
            .findings
            .iter()
            .map(|finding| (finding.rule.as_str(), finding.id.as_str()))
            .collect::<Vec<_>>();
        let identity = format!("repository:file:kd-retired-tree-{name}");
        let expected = if diagnostic.is_empty() {
            vec![]
        } else {
            vec![(identity.as_str(), diagnostic)]
        };
        assert_eq!(actual, expected, "intended retired-tree diagnostic");
        assert!(
            result
                .policies
                .iter()
                .all(|policy| policy.claim == "physical-paths" || policy.claim == "raw-utf8-text")
        );
    }
}

#[test]
fn retired_non_directory_proposal_preserves_all_ordinary_predicates() {
    assert_eq!(super::cases::run(&Fixture::new(), &policies()).len(), 144);
}

#[test]
fn retired_missing_empty_and_pruned_trees_have_explicit_outcomes() {
    for name in NAMES {
        for case in ["missing", "empty", "directory.rs", "git-empty", "git-rust"] {
            let fixture = Fixture::new();
            let tree = fixture.root.join("src/reactor").join(name);
            match case {
                "missing" => {}
                "empty" => fs::create_dir_all(&tree).expect("empty tree"),
                "directory.rs" => fs::create_dir_all(tree.join("empty.rs")).expect("directory"),
                _ => {
                    fs::create_dir_all(tree.join(".git")).expect("nested Git directory");
                    if case == "git-rust" {
                        fs::write(tree.join(".git/hidden.rs"), "// source").expect("hidden source");
                    }
                }
            }
            check(
                &fixture,
                name,
                Some(case != "git-rust"),
                if case.starts_with("git-") {
                    "REP-FILE-006"
                } else {
                    ""
                },
            );
        }
    }
}

#[cfg(unix)]
#[path = "retired_boundary_unix_test.rs"]
mod unix;
