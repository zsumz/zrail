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

fn fact(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        span: None,
        quality,
    }
}
