//! Internal Cargo dependency closure invariants.

use crate::cargo::{CrateRootAuthority, Dependency, DependencyKind, DependencySource, Package};

use super::implementation_packages;

#[test]
fn follows_workspace_and_repository_path_dependencies_transitively() {
    let packages = vec![
        package(
            "macros",
            "macros",
            vec![dependency(
                "helper",
                DependencySource::WorkspaceMember {
                    directory: "helper".into(),
                    requirement: None,
                },
            )],
        ),
        package(
            "helper",
            "helper",
            vec![dependency(
                "support",
                DependencySource::RepositoryPath {
                    path: "support".into(),
                    requirement: None,
                },
            )],
        ),
        package("support", "support", Vec::new()),
    ];

    let closure = implementation_packages(&packages, &packages[0]).expect("dependency closure");
    let directories = closure
        .iter()
        .map(|package| package.directory.as_str())
        .collect::<Vec<_>>();

    assert_eq!(directories, ["helper", "macros", "support"]);
}

#[test]
fn internal_dependency_with_no_package_fails_closed() {
    let packages = vec![package(
        "macros",
        "macros",
        vec![dependency(
            "missing",
            DependencySource::RepositoryPath {
                path: "missing".into(),
                requirement: None,
            },
        )],
    )];

    let error = implementation_packages(&packages, &packages[0])
        .expect_err("missing internal package must fail");

    assert!(error.to_string().contains("unavailable internal path"));
}

#[test]
fn internal_dependency_cycles_are_deduplicated() {
    let packages = vec![
        package(
            "macros",
            "macros",
            vec![dependency(
                "helper",
                DependencySource::WorkspaceMember {
                    directory: "helper".into(),
                    requirement: None,
                },
            )],
        ),
        package(
            "helper",
            "helper",
            vec![dependency(
                "macros",
                DependencySource::WorkspaceMember {
                    directory: "macros".into(),
                    requirement: None,
                },
            )],
        ),
    ];

    let closure = implementation_packages(&packages, &packages[0]).expect("bounded cycle");

    assert_eq!(closure.len(), 2);
}

fn package(name: &str, directory: &str, dependencies: Vec<Dependency>) -> Package {
    Package {
        name: name.into(),
        edition: "2024".into(),
        directory: directory.into(),
        dependencies,
        targets: Vec::new(),
    }
}

fn dependency(name: &str, source: DependencySource) -> Dependency {
    Dependency {
        alias: name.into(),
        name: name.into(),
        explicit_package: false,
        crate_root: name.into(),
        crate_root_authority: CrateRootAuthority::InspectedLibrary,
        kind: DependencyKind::Normal,
        target: None,
        optional: false,
        default_features: true,
        features: Vec::new(),
        source,
    }
}
