//! Repository glob visibility narrows candidates without guessing external exports.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::super::{MacroCandidate, MacroDerivation, MacroExpansionFact, ObservedFact};

use super::{MacroVisibility, imported_candidate};

#[test]
fn repository_globs_remove_only_the_redundant_unresolved_spelling() {
    let mut invocation = MacroExpansionFact::with_candidates(
        fact("reviewed", AnalysisQuality::Unresolved),
        vec![
            MacroCandidate::pending(
                fact("reviewed", AnalysisQuality::Unresolved),
                false,
                MacroDerivation::Written,
            ),
            MacroCandidate::pending(
                fact("super::reviewed", AnalysisQuality::Conservative),
                false,
                MacroDerivation::GlobImport,
            ),
            MacroCandidate::pending(
                fact("crate::missing", AnalysisQuality::Conservative),
                false,
                MacroDerivation::GlobImport,
            ),
        ],
    );
    let local = BTreeSet::from(["reviewed"]);

    MacroVisibility::default().resolve(&mut invocation, "src/lib.rs", Some(&local));

    assert_eq!(invocation.candidates.len(), 1);
    assert_eq!(invocation.candidates[0].observation.name, "super::reviewed");
}

#[test]
fn unknown_or_external_names_remain_conservative() {
    let mut invocation = MacroExpansionFact::with_candidates(
        fact("reviewed", AnalysisQuality::Exact),
        vec![
            MacroCandidate::pending(
                fact("reviewed", AnalysisQuality::Exact),
                false,
                MacroDerivation::Written,
            ),
            MacroCandidate::pending(
                fact("dependency::reviewed", AnalysisQuality::Conservative),
                false,
                MacroDerivation::GlobImport,
            ),
        ],
    );

    MacroVisibility::default().resolve(&mut invocation, "src/lib.rs", Some(&BTreeSet::new()));

    assert_eq!(invocation.candidates.len(), 2);
}

#[test]
fn external_imports_do_not_borrow_same_leaf_local_definitions() {
    let local = BTreeSet::from(["reviewed"]);
    let import = super::super::MacroImportFact {
        name: "reviewed".into(),
        target: "dependency::reviewed".into(),
        quality: AnalysisQuality::Exact,
        re_export: false,
    };

    let candidate = imported_candidate(
        &fact("super::reviewed", AnalysisQuality::Conservative),
        &import,
        Some(&local),
    );

    assert_eq!(candidate.observation.name, "dependency::reviewed");
    assert_eq!(
        candidate.origins,
        vec![super::super::MacroOrigin::Pending {
            local_module: false
        }]
    );
}

fn fact(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        canonical: Vec::new(),
        span: None,
        quality,
        guard: crate::source::SyntaxGuard::Ordinary,
    }
}
