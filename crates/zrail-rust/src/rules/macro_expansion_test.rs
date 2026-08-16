//! Directly inspected compiler macros cannot be confused with local definitions.

use zrail_core::AnalysisQuality;

use crate::source::ObservedFact;

use super::{directly_inspected, reviewed_names};

#[test]
fn local_definitions_shadow_intrinsic_shortcuts() {
    let include = fact("include");
    let mut local_include = fact("include");
    local_include.quality = AnalysisQuality::Unresolved;

    assert!(directly_inspected(&include));
    assert!(!directly_inspected(&local_include));
    assert!(!directly_inspected(&fact("std::include")));
    assert!(!directly_inspected(&fact("core::concat")));
}

#[test]
fn arbitrary_expression_macros_are_never_assumed_inspected() {
    for name in ["assert", "format", "matches", "vec", "tokio::select"] {
        assert!(!directly_inspected(&fact(name)));
    }
}

#[test]
fn every_conservative_canonical_identity_requires_review() {
    let mut expansion = fact("runtime::select");
    expansion.canonical = vec!["async_std::select".into(), "tokio::select".into()];
    expansion.quality = AnalysisQuality::Conservative;
    let partial = std::collections::BTreeMap::from([("tokio::select", "reviewed")]);
    let complete = std::collections::BTreeMap::from([
        ("async_std::select", "reviewed"),
        ("tokio::select", "reviewed"),
    ]);

    assert!(reviewed_names(&expansion, &partial).is_empty());
    assert_eq!(reviewed_names(&expansion, &complete).len(), 2);
    expansion.quality = AnalysisQuality::Unresolved;
    assert!(reviewed_names(&expansion, &complete).is_empty());
}

#[test]
fn bare_local_macros_cannot_borrow_a_global_name_allowance() {
    let allowed = std::collections::BTreeMap::from([("panic", "standard panic")]);
    let mut local = fact("panic");
    local.quality = AnalysisQuality::Unresolved;

    assert!(reviewed_names(&local, &allowed).is_empty());
    assert_eq!(
        reviewed_names(
            &fact("local::panic"),
            &std::collections::BTreeMap::from([("local::panic", "local macro")]),
        ),
        ["local::panic"]
    );
}

fn fact(name: &str) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
    }
}
