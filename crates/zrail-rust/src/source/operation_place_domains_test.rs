//! Projected candidates retain guard-specific support within one physical file.

use std::collections::BTreeSet;

use zrail_core::{AnalysisQuality, SourceSpan};

use super::*;
use crate::source::{CfgPredicate, CompilationMode};

#[test]
fn projected_candidates_replace_physical_fallback_only_in_their_world() {
    let a = domain("a", ["a"]);
    let b = domain("b", ["b"]);
    let domains = BTreeSet::from([a.clone(), b.clone()]);
    let paths = vec![
        fact("State", SyntaxGuard::Ordinary, true),
        fact("crate::selected::AState", feature("a"), false),
    ];

    let candidates =
        canonical_candidates_at(&paths, span(), Some(&domains), &SyntaxGuard::Ordinary);

    let projected = candidate(&candidates, "crate::selected::AState");
    assert_eq!(projected.domains.keys().collect::<Vec<_>>(), [&a]);
    let fallback = candidate(&candidates, "State");
    assert_eq!(fallback.domains.keys().collect::<Vec<_>>(), [&b]);
}

#[test]
fn operation_and_path_guards_intersect_on_exact_domain_identity() {
    let a = domain("a", ["a"]);
    let b = domain("b", ["b"]);
    let domains = BTreeSet::from([a.clone(), b]);
    let paths = vec![
        fact("crate::selected::AState", feature("a"), false),
        fact("crate::selected::BState", feature("b"), false),
    ];

    let candidates = candidates_at(&paths, span(), Some(&domains), &feature("a"));

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].name, "crate::selected::AState");
    assert_eq!(candidates[0].domains.keys().collect::<Vec<_>>(), [&a]);
    assert_eq!(candidates[0].domains[&a].quality, AnalysisQuality::Exact);
}

fn fact(name: &str, guard: SyntaxGuard, physical: bool) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: physical.then(|| "State".into()),
        implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
        canonical: Vec::new(),
        span: Some(span()),
        quality: AnalysisQuality::Exact,
        guard,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Type,
        generic_shadow: None,
    }
}

fn candidate<'a>(candidates: &'a [Candidate], name: &str) -> &'a Candidate {
    candidates
        .iter()
        .find(|candidate| candidate.name == name)
        .expect("candidate")
}

fn feature(name: &str) -> SyntaxGuard {
    SyntaxGuard::from_predicate(CfgPredicate::Feature(name.into()))
}

fn domain(name: &str, features: impl IntoIterator<Item = &'static str>) -> CompilationDomain {
    CompilationDomain {
        package: "fixture".into(),
        edition: "2024".into(),
        target: "fixture".into(),
        mode: CompilationMode::Library,
        feature_world: Some(name.into()),
        active_features: features.into_iter().map(str::to_owned).collect(),
    }
}

const fn span() -> SourceSpan {
    SourceSpan {
        line: 1,
        column: 0,
        end_line: 1,
        end_column: 5,
    }
}
