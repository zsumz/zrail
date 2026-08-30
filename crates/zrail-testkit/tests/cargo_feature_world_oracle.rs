//! Trusted Cargo output bounds zrail's deliberately conservative feature worlds.

use std::{collections::BTreeSet, path::Path};

use zrail_rust::governed_surface_report;

#[path = "cargo_feature_world_oracle/fixture.rs"]
mod fixture;

use fixture::{Fixture, Scenario};

#[test]
fn feature_empty_split_contexts_are_accepted() {
    for resolver in ["1", "2", "3"] {
        for scenario in [
            Scenario::EmptyDirect,
            Scenario::EmptyTransitive,
            Scenario::EmptyProcMacro,
        ] {
            let fixture = Fixture::new(resolver, scenario);
            assert_eq!(fixture.cargo_feature_sets(), sets(&[&[]]));
            let report = governed_surface_report(&fixture.root, Path::new("zrail.toml"))
                .expect("feature-empty split contexts stay inside the supported subset");
            let shared = report.feature_worlds[0]
                .packages
                .iter()
                .find(|package| package.package == "oracle-shared")
                .expect("shared package is governed");
            assert!(shared.active.is_empty());
        }
    }
}

#[test]
fn both_direct_split_directions_are_rejected() {
    for resolver in ["1", "2", "3"] {
        for scenario in [Scenario::HostOnlyDirect, Scenario::TargetOnlyDirect] {
            let fixture = Fixture::new(resolver, scenario);
            assert_eq!(
                fixture.cargo_feature_sets(),
                split_sets(resolver),
                "resolver {resolver} scenario {scenario:?}"
            );
            let error = rejection(&fixture);
            assert!(error.contains("package \"oracle-shared\""));
            assert!(error.contains("feature \"context\""));
            assert!(error.contains("build dependency edge from package \"oracle-app\""));
        }
    }
}

#[test]
fn transitive_build_and_proc_macro_host_splits_are_rejected() {
    for resolver in ["1", "2", "3"] {
        for scenario in [Scenario::HostOnlyTransitive, Scenario::HostOnlyProcMacro] {
            let fixture = Fixture::new(resolver, scenario);
            assert_eq!(fixture.cargo_feature_sets(), split_sets(resolver));
            let error = rejection(&fixture);
            assert!(error.contains("package \"oracle-shared\" feature \"context\""));
            assert!(error.contains("normal dependency edge from package \"oracle-helper\""));
            match scenario {
                Scenario::HostOnlyTransitive => {
                    assert!(error.contains("destination of the build dependency edge"));
                }
                Scenario::HostOnlyProcMacro => {
                    assert!(error.contains("from package \"oracle-macros\""));
                    assert!(error.contains("Cargo proc-macro host target"));
                }
                _ => panic!("only host scenarios are selected"),
            }
        }
    }
}

#[test]
fn cargo_convergent_nonempty_contexts_are_conservatively_rejected() {
    for resolver in ["1", "2", "3"] {
        for scenario in [
            Scenario::MaskedDirect,
            Scenario::MaskedTransitive,
            Scenario::MaskedProcMacro,
        ] {
            let fixture = Fixture::new(resolver, scenario);
            assert_eq!(fixture.cargo_feature_sets(), sets(&[&["context"]]));
            assert_strict_nonempty_rejection(&rejection(&fixture));
        }
    }
}

#[test]
fn authored_selected_and_default_features_cannot_mask_split_contexts() {
    for resolver in ["1", "2", "3"] {
        for scenario in [Scenario::SelectedDirect, Scenario::DefaultDirect] {
            let fixture = Fixture::new(resolver, scenario);
            let expected = match scenario {
                Scenario::SelectedDirect => sets(&[&["context"]]),
                Scenario::DefaultDirect => sets(&[&["context", "default"]]),
                _ => panic!("only authored masking scenarios are selected"),
            };
            assert_eq!(fixture.cargo_feature_sets(), expected);
            assert_strict_nonempty_rejection(&rejection(&fixture));
        }
    }
}

fn rejection(fixture: &Fixture) -> String {
    governed_surface_report(&fixture.root, Path::new("zrail.toml"))
        .expect_err("context-split nonempty package must be rejected")
        .to_string()
}

fn assert_strict_nonempty_rejection(error: &str) {
    assert!(error.contains("context-split package \"oracle-shared\""));
    assert!(error.contains("active feature \"context\""));
    assert!(error.contains("upper active feature set is empty"));
    assert!(
        error.contains("build dependency edge from package \"oracle-app\"")
            || error.contains("Cargo proc-macro host target")
    );
}

fn split_sets(resolver: &str) -> BTreeSet<Vec<String>> {
    if resolver == "1" {
        sets(&[&["context"]])
    } else {
        sets(&[&[], &["context"]])
    }
}

fn sets(values: &[&[&str]]) -> BTreeSet<Vec<String>> {
    values
        .iter()
        .map(|features| {
            features
                .iter()
                .map(|feature| (*feature).to_owned())
                .collect()
        })
        .collect()
}
