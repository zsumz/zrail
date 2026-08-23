//! Macro origin follows resolved repository and dependency identity, never path spelling.

use zrail_core::AnalysisQuality;

use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};

use super::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin, resolve};
use crate::source::ObservedFact;

#[test]
fn module_qualified_macro_is_repository_owned() {
    let package = package(Vec::new());
    let mut expansion = pending("helpers::reviewed", true);

    resolve(&mut expansion, &[&package]);

    assert_eq!(
        expansion.candidates[0].origins,
        [MacroOrigin::Repository {
            package: "app".into(),
            directory: ".".into(),
        }]
    );
}

#[test]
fn resolved_compiler_derive_uses_its_written_builtin_identity() {
    let observation = observed("Debug");
    let mut expansion = MacroExpansionFact::with_candidates(
        observation,
        vec![MacroCandidate::pending(
            observed("std::fmt::Debug"),
            false,
            MacroDerivation::ExactImport,
        )],
    );
    expansion.mark_builtin_derive_syntax();

    resolve(&mut expansion, &[]);

    assert_eq!(expansion.candidates.len(), 1);
    assert_eq!(expansion.candidates[0].observation.name, "Debug");
    assert_eq!(
        expansion.candidates[0].origins,
        [MacroOrigin::CompilerBuiltin]
    );
}

#[test]
fn workspace_dependency_macro_is_repository_owned() {
    let package = package(vec![dependency(
        "workspace_macros",
        DependencySource::WorkspaceMember {
            directory: "crates/macros".into(),
            requirement: None,
        },
    )]);
    let mut expansion = pending("workspace_macros::reviewed", false);

    resolve(&mut expansion, &[&package]);

    assert!(matches!(
        expansion.candidates[0].origins.as_slice(),
        [MacroOrigin::Repository { package, directory }]
            if package == "workspace-macros" && directory == "crates/macros"
    ));
}

#[test]
fn package_crate_root_is_repository_owned_for_integration_targets() {
    let package = package(Vec::new());
    let mut expansion = pending("app::reviewed", false);

    resolve(&mut expansion, &[&package]);

    assert_eq!(
        expansion.candidates[0].origins,
        [MacroOrigin::Repository {
            package: "app".into(),
            directory: ".".into(),
        }]
    );
}

#[test]
fn external_package_named_local_remains_external() {
    let package = package(vec![dependency(
        "local",
        DependencySource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        },
    )]);
    let mut expansion = pending("local::reviewed", false);

    resolve(&mut expansion, &[&package]);

    assert!(matches!(
        expansion.candidates[0].origins.as_slice(),
        [MacroOrigin::External { package, .. }] if package == "workspace-macros"
    ));
}

#[test]
fn excessive_source_identities_fail_closed() {
    let dependencies = (0..=super::MAX_MACRO_ORIGINS)
        .map(|index| {
            let mut dependency = dependency(
                "runtime",
                DependencySource::Registry {
                    registry: None,
                    index: None,
                    requirement: format!("={index}.0.0"),
                },
            );
            dependency.name = format!("runtime-{index}");
            dependency
        })
        .collect();
    let package = package(dependencies);
    let mut expansion = pending("runtime::select", false);

    resolve(&mut expansion, &[&package]);

    assert_eq!(expansion.candidates[0].origins, [MacroOrigin::Unresolved]);
}

#[test]
fn bounded_local_macro_definition_retains_repository_origin() {
    let package = package(Vec::new());
    let mut expansion = pending("reviewed", true);
    expansion.candidates[0].observation.quality = AnalysisQuality::Unresolved;

    resolve(&mut expansion, &[&package]);

    assert_eq!(
        expansion.candidates[0].origins,
        [MacroOrigin::Repository {
            package: "app".into(),
            directory: ".".into(),
        }]
    );
    assert_eq!(
        expansion.candidates[0].observation.quality,
        AnalysisQuality::Conservative
    );
}

#[test]
fn local_standard_name_shadow_remains_unresolved() {
    let package = package(Vec::new());
    let mut expansion = pending("panic", true);
    expansion.candidates[0].observation.quality = AnalysisQuality::Unresolved;

    resolve(&mut expansion, &[&package]);

    assert_eq!(expansion.candidates[0].origins, [MacroOrigin::Unresolved]);
}

fn pending(name: &str, local_module: bool) -> MacroExpansionFact {
    MacroExpansionFact::pending(observed(name), local_module)
}

fn observed(name: &str) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
        guard: crate::source::SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
    }
}

fn package(dependencies: Vec<Dependency>) -> Package {
    Package {
        name: "app".into(),
        directory: ".".into(),
        dependencies,
        targets: Vec::new(),
    }
}

fn dependency(alias: &str, source: DependencySource) -> Dependency {
    Dependency {
        alias: alias.into(),
        name: "workspace-macros".into(),
        explicit_package: true,
        crate_root: alias.into(),
        crate_root_authority: CrateRootAuthority::DeclaredAlias,
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: true,
        features: Vec::new(),
        source,
    }
}
