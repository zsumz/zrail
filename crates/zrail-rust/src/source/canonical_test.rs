//! Cargo roots retain observed spelling and canonicalize every policy candidate.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::AnalysisQuality;

use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};

use super::{
    MAX_IDENTITIES_PER_ROOT, ObservedFact, canonicalize_fact, canonicalize_fact_bounded,
    dependency_roots,
};

#[test]
fn renamed_dependency_roots_preserve_observed_diagnostics() {
    let mut fact = observed("runtime::process::Command", AnalysisQuality::Exact);
    canonicalize_fact(&mut fact, &roots(&[("runtime", &["tokio"])]));

    assert_eq!(fact.name, "runtime::process::Command");
    assert_eq!(fact.canonical, ["tokio::process::Command"]);
    assert_eq!(fact.quality, AnalysisQuality::Exact);
}

#[test]
fn raw_identifier_dependency_roots_recover_the_cargo_alias() {
    let mut observed = observed("r#async::spawn", AnalysisQuality::Exact);
    let roots = BTreeMap::from([("async".into(), BTreeSet::from(["smol".into()]))]);

    canonicalize_fact(&mut observed, &roots);

    assert_eq!(observed.canonical, ["smol::spawn"]);
    assert_eq!(observed.name, "r#async::spawn");
}

#[test]
fn target_dependent_aliases_remain_conservative_candidates() {
    let mut fact = observed("runtime::spawn", AnalysisQuality::Exact);
    canonicalize_fact(&mut fact, &roots(&[("runtime", &["async_std", "tokio"])]));

    assert_eq!(fact.canonical, ["async_std::spawn", "tokio::spawn"]);
    assert_eq!(fact.quality, AnalysisQuality::Conservative);
}

#[test]
fn canonical_identity_replaces_the_alias_as_policy_authority() {
    let mut fact = observed("tokio::spawn", AnalysisQuality::Exact);
    canonicalize_fact(&mut fact, &roots(&[("tokio", &["benign_runtime"])]));

    assert_eq!(
        fact.policy_names().collect::<Vec<_>>(),
        ["benign_runtime::spawn"]
    );
}

#[test]
fn excessive_shared_alias_identities_fail_closed_without_a_cross_product() {
    let packages = (0..=MAX_IDENTITIES_PER_ROOT)
        .map(|index| package(index, "runtime"))
        .collect::<Vec<_>>();
    let selected = packages.iter().collect::<Vec<_>>();

    let (roots, overflowed) = dependency_roots(&selected, &BTreeSet::from(["runtime".into()]));
    let mut fact = observed("runtime::spawn", AnalysisQuality::Exact);
    canonicalize_fact_bounded(&mut fact, &roots, &overflowed);

    assert_eq!(roots["runtime"].len(), MAX_IDENTITIES_PER_ROOT);
    assert!(overflowed.contains("runtime"));
    assert_eq!(fact.quality, AnalysisQuality::Unresolved);
    assert!(fact.canonical.is_empty());
}

fn observed(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
        canonical: Vec::new(),
        span: None,
        quality,
        guard: crate::source::SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: crate::source::FactNamespace::Unknown,
        generic_shadow: None,
    }
}

fn roots(entries: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    entries
        .iter()
        .map(|(alias, names)| {
            (
                (*alias).into(),
                names.iter().map(|name| (*name).into()).collect(),
            )
        })
        .collect()
}

fn package(index: usize, alias: &str) -> Package {
    Package {
        name: format!("package-{index}"),
        edition: "2024".into(),
        directory: format!("crates/package-{index}"),
        dependencies: vec![Dependency {
            alias: alias.into(),
            name: format!("runtime-{index}"),
            explicit_package: true,
            crate_root: alias.into(),
            crate_root_authority: CrateRootAuthority::DeclaredAlias,
            kind: DependencyKind::Normal,
            target: None,
            optional: false,
            default_features: true,
            features: Vec::new(),
            source: DependencySource::Registry {
                registry: None,
                index: None,
                requirement: "1".into(),
            },
        }],
        targets: Vec::new(),
    }
}
