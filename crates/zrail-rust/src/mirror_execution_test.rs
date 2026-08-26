//! Mirror feature identity selects one exact compilation world without Cargo execution.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{AnalysisQuality, TestExecutionIdentity, TestMirrorContract};

use super::validate_inputs;
use crate::{
    cargo::{
        CargoTarget, CargoTargetKind, CargoWorkspace, Package, PackageFeatureSet,
        ResolvedFeatureWorld, ResolvedPackageFeatures,
    },
    source::{
        CfgPredicate, CompilationDomain, CompilationMode, FactNamespace, ObservedFact, SyntaxGuard,
    },
};

#[test]
fn feature_gated_mirror_requires_one_matching_exact_world() {
    let cargo = workspace(Vec::new());
    let domains = domains("strict", ["strict"]);
    let declarations = vec![declaration(SyntaxGuard::from_predicate(
        CfgPredicate::Feature("strict".into()),
    ))];
    let worlds = vec![world("strict", true, ["strict"], ["strict"])];
    let policy = mirror(true, vec!["strict".into()]);

    validate_inputs(
        &policy,
        &cargo,
        &worlds,
        &domains,
        "tests/proof.rs",
        &declarations,
    )
    .expect("exact feature world");

    let error = validate_inputs(
        &mirror(false, Vec::new()),
        &cargo,
        &worlds,
        &domains,
        "tests/proof.rs",
        &declarations,
    )
    .expect_err("unmatched execution identity");
    assert!(error.contains("matching []"));
}

#[test]
fn configured_world_selection_is_rejected_when_ambiguous() {
    let cargo = workspace(Vec::new());
    let worlds = vec![
        world("first", true, ["strict"], ["strict"]),
        world("second", true, ["strict"], ["strict"]),
    ];
    let error = validate_inputs(
        &mirror(true, vec!["strict".into()]),
        &cargo,
        &worlds,
        &domains("first", ["strict"]),
        "tests/proof.rs",
        &[declaration(SyntaxGuard::Ordinary)],
    )
    .expect_err("ambiguous worlds");

    assert!(error.contains("first, second"));
}

#[test]
fn legacy_execution_identity_enforces_target_required_features() {
    let cargo = workspace(vec!["strict".into()]);
    let error = validate_inputs(
        &mirror(false, Vec::new()),
        &cargo,
        &[],
        &domains("legacy", []),
        "tests/proof.rs",
        &[declaration(SyntaxGuard::Ordinary)],
    )
    .expect_err("disabled test target");

    assert!(error.contains("no enabled Cargo test target"));
}

fn workspace(required_features: Vec<String>) -> CargoWorkspace {
    let manifest = toml::from_str::<toml::Value>("[features]\ndefault = []\nstrict = []\n")
        .expect("feature manifest");
    CargoWorkspace {
        declared_members: vec![".".into()],
        observed_members: vec![".".into()],
        packages: vec![Package {
            name: "app".into(),
            edition: "2024".into(),
            directory: ".".into(),
            dependencies: Vec::new(),
            targets: vec![CargoTarget {
                name: "proof".into(),
                path: "tests/proof.rs".into(),
                kind: CargoTargetKind::Test,
                required_features,
            }],
        }],
        package_features: BTreeMap::from([(
            "app".into(),
            PackageFeatureSet::parse(&manifest, &[]).expect("parse features"),
        )]),
        authority_surfaces: Vec::new(),
        manifest_scopes: BTreeMap::new(),
    }
}

fn world<const S: usize, const A: usize>(
    name: &str,
    default_features: bool,
    selected: [&str; S],
    active: [&str; A],
) -> ResolvedFeatureWorld {
    ResolvedFeatureWorld {
        name: name.into(),
        packages: BTreeMap::from([(
            "app".into(),
            ResolvedPackageFeatures {
                default_features,
                selected: selected.into_iter().map(str::to_owned).collect(),
                active: active.into_iter().map(str::to_owned).collect(),
            },
        )]),
    }
}

fn domains<const N: usize>(
    world: &str,
    active: [&str; N],
) -> BTreeMap<String, BTreeSet<CompilationDomain>> {
    BTreeMap::from([(
        "tests/proof.rs".into(),
        BTreeSet::from([CompilationDomain {
            package: "app".into(),
            edition: "2024".into(),
            target: "proof".into(),
            mode: CompilationMode::IntegrationTest,
            feature_world: (world != "legacy").then(|| world.into()),
            active_features: active.into_iter().map(str::to_owned).collect(),
        }]),
    )])
}

fn declaration(guard: SyntaxGuard) -> ObservedFact {
    ObservedFact {
        name: "proof".into(),
        written: None,
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
        guard,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Value,
    }
}

fn mirror(default_features: bool, features: Vec<String>) -> TestMirrorContract {
    TestMirrorContract {
        production: "src/lib.rs".into(),
        test: "tests/proof.rs".into(),
        name: "proof".into(),
        receipt: "evidence/proof.json".into(),
        inputs: vec!["Cargo.toml".into()],
        execution: TestExecutionIdentity {
            command: "cargo test -p app --test proof".into(),
            package: "app".into(),
            default_features,
            features,
            target: "x86_64-unknown-linux-gnu".into(),
            toolchain: "rustc 1.96.0".into(),
        },
        reason: "Exact mirror".into(),
    }
}
