//! Capability prefixes distinguish exact paths from conservative glob roots.

use zrail_core::AnalysisQuality;

use crate::source::ObservedFact;

use super::path_matches;

#[test]
fn denied_paths_cover_descendants_but_not_similar_names() {
    assert!(path_matches(
        "std::net",
        &fact("std::net::TcpStream", AnalysisQuality::Exact)
    ));
    assert!(!path_matches(
        "std::net",
        &fact("my_std::net", AnalysisQuality::Exact)
    ));
}

#[test]
fn conservative_glob_roots_cover_narrower_denials() {
    assert!(path_matches(
        "std::time::Instant",
        &fact("std::time", AnalysisQuality::Conservative)
    ));
    assert!(!path_matches(
        "std::time::Instant",
        &fact("std::thread", AnalysisQuality::Conservative)
    ));
}

#[test]
fn canonical_dependency_identity_owns_policy_matching() {
    let mut renamed = fact("runtime::spawn", AnalysisQuality::Exact);
    renamed.canonical.push("tokio::spawn".into());

    assert!(path_matches("tokio", &renamed));
    assert!(!path_matches("runtime", &renamed));
}

#[test]
fn raw_identifiers_cannot_change_policy_identity() {
    assert!(path_matches(
        "std::process",
        &fact("r#std::r#process::Command", AnalysisQuality::Exact)
    ));
}

fn fact(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        canonical: Vec::new(),
        span: None,
        quality,
        guard: crate::source::SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: crate::source::FactNamespace::Unknown,
    }
}
