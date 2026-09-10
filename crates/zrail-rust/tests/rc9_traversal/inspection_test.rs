//! Corrected inspection is publicly loaded before reusing every physical boundary fixture.

use super::model::{self, Fixture, Row};
use std::{fs, path::Path};
use zrail_core::{RepositoryFilePredicate, RepositoryFileRule, load_contract};

const POLICY: &str = "docs/rc9/policies/kafka-driver.traversal-inspection.fragment.toml";

fn source() -> toml::Value {
    let mut base: toml::Value = toml::from_str(
        &fs::read_to_string(
            model::project().join("crates/zrail-testkit/tests/fixtures/good/zrail.toml"),
        )
        .expect("complete scaffold"),
    )
    .expect("scaffold TOML");
    let fragment: toml::Value = toml::from_str(
        &fs::read_to_string(model::project().join(POLICY)).expect("corrected fragment"),
    )
    .expect("fragment TOML");
    base["repository"]
        .as_table_mut()
        .expect("repository table")
        .insert("files".into(), fragment["repository"]["files"].clone());
    base
}

fn load(fixture: &Fixture, value: &toml::Value) -> Result<Vec<RepositoryFileRule>, String> {
    fixture.write("zrail.toml", toml::to_string(value).expect("carrier TOML"));
    load_contract(&fixture.root, Path::new("zrail.toml"))
        .map(|loaded| loaded.contract.repository.files)
        .map_err(|error| error.to_string())
}

fn observe(fixture: &Fixture, case: &str, old_error: Option<&str>, diagnostic: &str) -> Row {
    let policies = load(fixture, &source()).expect("publicly loadable inspection");
    let row = model::observe_with(fixture, case, old_error, diagnostic, &policies);
    if row.native_error.is_none() {
        let inspection = row
            .observations
            .iter()
            .find(|rule| rule.policy_id == "repository:file:kd-source-traversal")
            .expect("inspection coverage");
        assert_eq!(inspection.claim, "physical-entry-inspection");
        assert_eq!(inspection.policy, policies[1]);
        assert!(inspection.satisfied);
        assert!(
            inspection
                .entries
                .iter()
                .all(|entry| entry.sha256.is_none() && entry.bytes.is_none())
        );
    }
    row
}

#[test]
fn inspection_contract_is_closed_and_vacuous_counts_stay_invalid() {
    let fixture = Fixture::new();
    let valid = source();
    let policies = load(&fixture, &valid).expect("loadable corrected fragment");
    let mut historical = model::policies();
    historical[1].predicate = RepositoryFilePredicate::Inspect {};
    assert_eq!(
        historical, policies,
        "only the predicate representation changes"
    );
    for entry in [None, Some("file"), Some("directory"), Some("non-directory")] {
        let mut invalid = valid.clone();
        let rule = invalid["repository"]["files"][1]
            .as_table_mut()
            .expect("rule table");
        match entry {
            Some(entry) => {
                rule.insert("entry".into(), toml::Value::String(entry.into()));
            }
            None => {
                rule.remove("entry");
            }
        }
        assert!(
            load(&fixture, &invalid)
                .expect_err("any required")
                .contains("must select any entries")
        );
    }
    for field in ["minimum", "maximum", "paths", "unknown"] {
        let mut invalid = valid.clone();
        invalid["repository"]["files"][1]["predicate"]
            .as_table_mut()
            .expect("predicate")
            .insert(field.into(), toml::Value::Integer(0));
        assert!(
            load(&fixture, &invalid)
                .expect_err("closed inspection")
                .contains("unknown field")
        );
    }
    let mut count = valid.clone();
    count["repository"]["files"][1]["predicate"] =
        toml::Value::try_from(&model::policies()[1].predicate).expect("historical predicate");
    assert!(
        load(&fixture, &count)
            .expect_err("not an inspection alias")
            .contains("vacuous count constraint")
    );
    count["repository"]["files"][1]["predicate"]
        .as_table_mut()
        .expect("count")
        .insert("maximum".into(), toml::Value::Integer(0));
    assert!(
        load(&fixture, &count).is_ok(),
        "zero maximum remains a valid prohibition"
    );
}

#[test]
fn inspection_loaded_contract_covers_roots_empty_trees_and_selection() {
    assert_eq!(super::portable_with(observe).len(), 48);
}

#[test]
fn inspection_binds_entries_and_policy_but_does_not_claim_file_contents() {
    let fixture = Fixture::new();
    let policies = load(&fixture, &source()).expect("load");
    let analyze = |rules: &[RepositoryFileRule]| {
        crate::repository_files::analyze(&fixture.root, rules).expect("complete")
    };
    let empty = analyze(&policies);
    assert_eq!(empty.policies[1].entries.len(), model::ROOTS.len());
    assert!(
        empty.policies[1]
            .entries
            .iter()
            .all(|entry| entry.kind == "directory")
    );
    fixture.write("src/visible.rs", "// first bytes");
    let populated = analyze(&policies);
    assert_ne!(empty.binding_sha256, populated.binding_sha256);
    fixture.write("src/visible.rs", "// different bytes with no content claim");
    assert_eq!(populated.binding_sha256, analyze(&policies).binding_sha256);
    let mut excluded = policies.clone();
    excluded[1].exclude.push("src/visible.rs".into());
    assert_ne!(populated.binding_sha256, analyze(&excluded).binding_sha256);
}

#[cfg(unix)]
#[test]
fn inspection_loaded_contract_covers_links_and_special_entries() {
    assert_eq!(super::unix::links_and_specials_with(observe).len(), 66);
}

#[cfg(unix)]
#[test]
#[ignore = "requires Unix permission enforcement, explicitly checked before qualification"]
fn inspection_loaded_contract_covers_unread_boundaries() {
    assert_eq!(super::unix::permissions_with(observe).len(), 19);
}
