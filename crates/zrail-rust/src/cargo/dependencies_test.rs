//! Dependency extraction retains every architecture-relevant identity field.

use std::collections::BTreeMap;

use toml::Value;

use super::super::model::{DependencyKind, DependencySource};
use super::{collect_dependencies, workspace_dependencies};

#[test]
fn aliases_and_workspace_sources_are_preserved() {
    let root = r#"
        [workspace.dependencies]
        local = { package = "local-package", path = "crates/local", features = ["root"] }
    "#
    .parse::<Value>()
    .expect("parse root manifest");
    let member = r#"
        [dependencies]
        renamed = { package = "real-package", version = "1" }
        local = { workspace = true, optional = true, features = ["member"] }
    "#
    .parse::<Value>()
    .expect("parse member manifest");
    let workspace = workspace_dependencies(&root).expect("workspace dependencies");

    let dependencies =
        collect_dependencies(&member, &workspace, "crates/app").expect("collect dependencies");

    let renamed = dependencies
        .iter()
        .find(|dependency| dependency.alias == "renamed")
        .expect("renamed dependency");
    assert_eq!(renamed.name, "real-package");
    assert!(matches!(renamed.source, DependencySource::Registry { .. }));
    let local = dependencies
        .iter()
        .find(|dependency| dependency.alias == "local")
        .expect("local dependency");
    assert_eq!(local.name, "local-package");
    assert!(local.optional);
    assert_eq!(local.features, ["member", "root"]);
    assert_eq!(
        local.source,
        DependencySource::RepositoryPath {
            path: "crates/local".into(),
            requirement: None,
        }
    );
}

#[test]
fn path_version_requirements_are_preserved() {
    let manifest = "[dependencies]\nlocal = { path = '../local', version = '2' }"
        .parse::<Value>()
        .expect("parse manifest");

    let dependencies = collect_dependencies(&manifest, &BTreeMap::new(), "crates/app")
        .expect("collect path dependency");

    assert!(matches!(
        dependencies[0].source,
        DependencySource::RepositoryPath {
            ref path,
            requirement: Some(ref requirement),
        } if path == "crates/local" && requirement == "2"
    ));
}

#[test]
fn registry_git_target_and_feature_identity_are_distinct() {
    let manifest = r#"
        [dependencies]
        registry = { package = "shared", version = "1", registry = "private", default-features = false }
        git = { package = "shared", git = "https://example.test/shared", rev = "abc", version = "2" }

        [target.'cfg(unix)'.dependencies]
        registry = { package = "shared", version = "1", features = ["unix"] }
    "#
    .parse::<Value>()
    .expect("parse manifest");

    let dependencies = collect_dependencies(&manifest, &BTreeMap::new(), ".")
        .expect("collect source-aware dependencies");

    assert_eq!(dependencies.len(), 3);
    assert!(dependencies.iter().any(|dependency| {
        dependency.alias == "registry"
            && !dependency.default_features
            && matches!(
                dependency.source,
                DependencySource::Registry {
                    registry: Some(ref name),
                    ..
                } if name == "private"
            )
    }));
    assert!(dependencies.iter().any(|dependency| {
        dependency.alias == "git"
            && matches!(
                dependency.source,
                DependencySource::Git {
                    rev: Some(ref rev),
                    ..
                } if rev == "abc"
            )
    }));
    assert!(dependencies.iter().any(|dependency| {
        dependency.target.as_deref() == Some("cfg(unix)") && dependency.features == ["unix"]
    }));
}

#[test]
fn dependency_tables_and_specs_fail_closed() {
    for source in [
        "dependencies = []",
        "[dependencies]\nserde = 1",
        "[dependencies]\nserde = { workspace = true }",
        "[dependencies]\nserde = { git = 'x', path = 'y' }",
        "[dependencies]\nserde = { git = 'x', branch = 'a', rev = 'b' }",
        "[dependencies]\nserde = { path = '../../../outside' }",
        "[dependencies]\nserde = { version = '1', public = true }",
        "[target]\n'cfg(unix)' = 1",
    ] {
        let manifest = source.parse::<Value>().expect("parse manifest");
        assert!(
            collect_dependencies(&manifest, &BTreeMap::new(), "crates/member").is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn workspace_dependency_tables_are_strict() {
    let manifest = "[workspace]\ndependencies = []"
        .parse::<Value>()
        .expect("parse manifest");

    assert!(workspace_dependencies(&manifest).is_err());

    let optional = "[workspace.dependencies]\nserde = { version = '1', optional = false }"
        .parse::<Value>()
        .expect("parse optional workspace dependency");
    assert!(workspace_dependencies(&optional).is_err());

    let root = "[workspace.dependencies]\nserde = '1'"
        .parse::<Value>()
        .expect("parse workspace dependency");
    let member = "[dependencies]\nserde = { workspace = true, default-features = false }"
        .parse::<Value>()
        .expect("parse inherited dependency");
    let workspace = workspace_dependencies(&root).expect("workspace dependencies");
    assert!(collect_dependencies(&member, &workspace, ".").is_err());
}

#[test]
fn kinds_remain_distinct() {
    let manifest = r#"
        [dependencies]
        same = "1"
        [dev-dependencies]
        same = "1"
        [build-dependencies]
        same = "1"
    "#
    .parse::<Value>()
    .expect("parse manifest");
    let dependencies =
        collect_dependencies(&manifest, &BTreeMap::new(), ".").expect("collect dependencies");

    assert_eq!(dependencies.len(), 3);
    for kind in [
        DependencyKind::Normal,
        DependencyKind::Development,
        DependencyKind::Build,
    ] {
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.kind == kind)
        );
    }
}
