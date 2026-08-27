//! Final field declarations retain exact feature-world support.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use super::*;
use crate::source::{CompilationDomain, CompilationMode, SyntaxGuard};

#[test]
fn declaring_field_is_missing_outside_its_feature_world() {
    let off = domain(&[]);
    let on = domain(&["extra"]);
    let exact = Support {
        quality: AnalysisQuality::Exact,
        projected: false,
    };
    let candidate = Candidate {
        name: "fixture::State".into(),
        domains: BTreeMap::from([(off.clone(), exact), (on.clone(), exact)]),
    };
    let declaration = super::super::catalog::Declaration {
        domains: candidate.domains.clone(),
        members: BTreeMap::from([(
            "epoch".into(),
            super::super::catalog::Member {
                domains: BTreeMap::from([(on.clone(), exact)]),
                guard: SyntaxGuard::Ordinary,
            },
        )]),
        fields: BTreeMap::new(),
    };
    let catalog = Catalog(BTreeMap::from([(
        candidate.name.clone(),
        vec![declaration],
    )]));

    let (declaring, missing) = super::declaring(&[candidate], "epoch", &catalog);

    assert_eq!(missing, BTreeSet::from([off]));
    assert_eq!(declaring.len(), 1);
    assert_eq!(declaring[0].name, "fixture::State::epoch");
    assert_eq!(declaring[0].domains.keys().collect::<Vec<_>>(), [&on]);
}

fn domain(features: &[&str]) -> CompilationDomain {
    CompilationDomain {
        package: "fixture".into(),
        edition: "2024".into(),
        target: "fixture".into(),
        mode: CompilationMode::Library,
        feature_world: Some("world".into()),
        active_features: features.iter().map(|feature| (*feature).into()).collect(),
    }
}
