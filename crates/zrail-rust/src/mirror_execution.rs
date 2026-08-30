//! Mirror execution identities must select one exact feature compilation world.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::TestMirrorContract;

use crate::{
    cargo::{CargoTargetKind, CargoWorkspace, ResolvedFeatureWorld},
    source::{CompilationDomain, CompilationMode, GuardAvailability, RustFileFacts},
};

pub(crate) fn validate(
    mirror: &TestMirrorContract,
    cargo: &CargoWorkspace,
    worlds: &[ResolvedFeatureWorld],
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    test: &RustFileFacts,
) -> Result<(), String> {
    validate_inputs(
        mirror,
        cargo,
        worlds,
        compilation_domains,
        &test.relative,
        &test.tests,
    )
}

fn validate_inputs(
    mirror: &TestMirrorContract,
    cargo: &CargoWorkspace,
    worlds: &[ResolvedFeatureWorld],
    compilation_domains: &BTreeMap<String, BTreeSet<CompilationDomain>>,
    test_path: &str,
    declarations: &[crate::source::ObservedFact],
) -> Result<(), String> {
    let mut selected = mirror.execution.features.clone();
    selected.sort();
    if selected.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("mirror execution features must be unique".into());
    }
    let package = cargo
        .packages
        .iter()
        .find(|package| package.name == mirror.execution.package)
        .ok_or_else(|| "mirror execution package is not a workspace package".to_owned())?;
    let (world_name, active) = if worlds.is_empty() {
        let active = cargo.package_features[&package.name]
            .resolve_details(mirror.execution.default_features, &selected)?
            .active;
        ("mirror-execution".to_owned(), active)
    } else {
        let matches = worlds
            .iter()
            .filter(|world| {
                let features = &world.packages[&package.name];
                features.default_features == mirror.execution.default_features
                    && features.selected == selected
            })
            .collect::<Vec<_>>();
        let [world] = matches.as_slice() else {
            let matching = matches
                .iter()
                .map(|world| world.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let configured = worlds
                .iter()
                .map(|world| world.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "mirror execution must match exactly one feature world; matching [{matching}], configured [{configured}]"
            ));
        };
        (
            world.name.clone(),
            world.packages[&package.name]
                .active
                .iter()
                .cloned()
                .collect(),
        )
    };
    let domains = compilation_domains
        .get(test_path)
        .into_iter()
        .flatten()
        .filter(|domain| domain.package == package.name && domain.mode.enables_cfg_test())
        .filter(|domain| worlds.is_empty() || domain.feature_world.as_deref() == Some(&world_name))
        .filter(|domain| target_enabled(package, domain, &active))
        .map(|domain| {
            let mut domain = domain.clone();
            domain.feature_world = Some(world_name.clone());
            domain.active_features.clone_from(&active);
            domain
        })
        .collect::<Vec<_>>();
    if domains.is_empty() {
        return Err(format!(
            "mirror test has no enabled Cargo test target in feature world {world_name:?}"
        ));
    }
    let declarations = declarations
        .iter()
        .filter(|fact| fact.name == mirror.name)
        .filter(|fact| {
            domains
                .iter()
                .any(|domain| fact.guard.availability_in_domain(domain) == GuardAvailability::Exact)
        })
        .count();
    if declarations != 1 {
        return Err(format!(
            "mirror test {:?} must be exactly present once in feature world {world_name:?}; found {declarations}",
            mirror.name
        ));
    }
    Ok(())
}

fn target_enabled(
    package: &crate::cargo::Package,
    domain: &CompilationDomain,
    active: &BTreeSet<String>,
) -> bool {
    package.targets.iter().any(|target| {
        target.name == domain.target
            && mode_matches(target.kind, domain.mode)
            && target
                .required_features
                .iter()
                .all(|feature| active.contains(feature))
    })
}

const fn mode_matches(kind: CargoTargetKind, mode: CompilationMode) -> bool {
    matches!(
        (kind, mode),
        (
            CargoTargetKind::Library,
            CompilationMode::Library | CompilationMode::LibraryTest
        ) | (
            CargoTargetKind::ProcMacro,
            CompilationMode::ProcMacro | CompilationMode::ProcMacroTest
        ) | (
            CargoTargetKind::Binary,
            CompilationMode::Binary | CompilationMode::BinaryTest
        ) | (CargoTargetKind::Test, CompilationMode::IntegrationTest)
            | (CargoTargetKind::Benchmark, CompilationMode::Benchmark)
            | (
                CargoTargetKind::Example,
                CompilationMode::Example | CompilationMode::ExampleTest
            )
            | (CargoTargetKind::BuildScript, CompilationMode::BuildScript)
    )
}

#[cfg(test)]
#[path = "mirror_execution_test.rs"]
mod mirror_execution_test;
