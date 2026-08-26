//! Cfg predicates evaluate feature atoms exactly and target atoms conservatively.

use std::collections::BTreeSet;

use syn::Meta;

use super::{CfgContext, CfgPredicate, CfgTruth};

#[test]
fn feature_boolean_algebra_is_exact_inside_a_world() {
    let predicate = predicate("all(feature = \"strict\", not(feature = \"trace\"))");
    let strict = BTreeSet::from(["strict".into()]);
    let both = BTreeSet::from(["strict".into(), "trace".into()]);

    assert_eq!(predicate.evaluate(&context(&strict)), CfgTruth::True);
    assert_eq!(predicate.evaluate(&context(&both)), CfgTruth::False);
    assert_eq!(
        predicate.evaluate(&CfgContext {
            test: false,
            active_features: None,
        }),
        CfgTruth::Unknown
    );
}

#[test]
fn known_feature_branches_reduce_unknown_target_predicates() {
    let predicate = predicate("any(feature = \"portable\", target_os = \"linux\")");
    let portable = BTreeSet::from(["portable".into()]);
    let empty = BTreeSet::new();

    assert_eq!(predicate.evaluate(&context(&portable)), CfgTruth::True);
    assert_eq!(predicate.evaluate(&context(&empty)), CfgTruth::Unknown);
}

#[test]
fn canonical_algebra_detects_inverse_predicates() {
    let feature = predicate("feature = \"strict\"");

    assert_eq!(
        CfgPredicate::all(vec![feature.clone(), CfgPredicate::not(feature.clone())]),
        CfgPredicate::False
    );
    assert_eq!(
        CfgPredicate::any(vec![feature.clone(), CfgPredicate::not(feature)]),
        CfgPredicate::True
    );
}

#[test]
fn opaque_target_predicates_are_span_free_and_structurally_equal() {
    let first = predicate("all(unix, target_os = \"linux\")");
    let second = predicate("all(unix, target_os = \"linux\")");

    assert_eq!(first, second);
    assert!(first.implies(&second));
}

#[test]
fn conjunction_implies_each_weaker_term() {
    let stronger = predicate("all(unix, target_os = \"linux\")");

    assert!(stronger.implies(&predicate("unix")));
    assert!(!predicate("unix").implies(&stronger));
}

#[test]
fn target_partition_is_proven_exhaustive_without_selecting_a_target() {
    let linux = predicate("target_os = \"linux\"");
    let macos = predicate("target_os = \"macos\"");
    let other = predicate("not(any(target_os = \"linux\", target_os = \"macos\"))");
    let partition = CfgPredicate::any(vec![linux, macos, other]);

    assert!(CfgPredicate::True.implies(&partition));
}

#[test]
fn distinct_single_valued_target_cfgs_cannot_overlap() {
    let operating_systems = CfgPredicate::all(vec![
        predicate("target_os = \"linux\""),
        predicate("target_os = \"macos\""),
    ]);
    let target_features = CfgPredicate::all(vec![
        predicate("target_feature = \"sse2\""),
        predicate("target_feature = \"avx\""),
    ]);

    assert_eq!(operating_systems.is_satisfiable(), Some(false));
    assert_eq!(target_features.is_satisfiable(), Some(true));
}

fn predicate(source: &str) -> CfgPredicate {
    CfgPredicate::from_meta(&syn::parse_str::<Meta>(source).expect("parse cfg meta"))
}

fn context(features: &BTreeSet<String>) -> CfgContext<'_> {
    CfgContext {
        test: false,
        active_features: Some(features),
    }
}
