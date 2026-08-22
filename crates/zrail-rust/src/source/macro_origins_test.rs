//! Macro origin follows resolved repository and dependency identity, never path spelling.

use zrail_core::AnalysisQuality;

use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};

use super::{MacroExpansionFact, MacroOrigin, resolve};
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

fn pending(name: &str, local_module: bool) -> MacroExpansionFact {
    MacroExpansionFact::pending(
        ObservedFact {
            name: name.into(),
            canonical: Vec::new(),
            span: None,
            quality: AnalysisQuality::Exact,
        },
        local_module,
    )
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
