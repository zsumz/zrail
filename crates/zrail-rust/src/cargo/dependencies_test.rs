//! Malformed and unresolved dependency declarations fail closed.

use std::collections::BTreeMap;

use toml::Value;

use super::super::model::DependencyKind;
use super::{
    collect_dependencies, collect_dependency_paths, workspace_dependencies,
    workspace_dependency_paths,
};

#[test]
fn aliases_resolve_to_real_package_names() {
    let manifest = r#"
        [dependencies]
        renamed = { package = "real-package", version = "1" }
        local = { workspace = true }
    "#
    .parse::<Value>()
    .expect("parse test manifest");
    let workspace = BTreeMap::from([("local".into(), "local-package".into())]);
    let dependencies = collect_dependencies(&manifest, &workspace).expect("collect dependencies");

    assert!(dependencies.iter().any(|dependency| {
        dependency.name == "real-package" && dependency.kind == DependencyKind::Normal
    }));
    assert!(
        dependencies
            .iter()
            .any(|dependency| dependency.name == "local-package")
    );
}

#[test]
fn dependency_tables_and_specs_are_strict() {
    let wrong_table = "dependencies = []"
        .parse::<Value>()
        .expect("parse manifest");
    assert!(collect_dependencies(&wrong_table, &BTreeMap::new()).is_err());

    let wrong_spec = "[dependencies]\nserde = 1"
        .parse::<Value>()
        .expect("parse manifest");
    assert!(collect_dependencies(&wrong_spec, &BTreeMap::new()).is_err());
}

#[test]
fn inherited_dependencies_must_exist_at_the_workspace_root() {
    let manifest = "[dependencies]\nmissing = { workspace = true }"
        .parse::<Value>()
        .expect("parse manifest");

    assert!(collect_dependencies(&manifest, &BTreeMap::new()).is_err());
}

#[test]
fn workspace_dependency_tables_are_strict() {
    let manifest = "[workspace]\ndependencies = []"
        .parse::<Value>()
        .expect("parse manifest");

    assert!(workspace_dependencies(&manifest).is_err());
}

#[test]
fn inherited_workspace_paths_retain_their_root_relative_base() {
    let root = "[workspace.dependencies]\nmember = { path = 'crates/member' }"
        .parse::<Value>()
        .expect("parse root");
    let member = "[dependencies]\nmember = { workspace = true }"
        .parse::<Value>()
        .expect("parse member");
    let workspace = workspace_dependency_paths(&root).expect("workspace paths");

    let paths = collect_dependency_paths(&member, &workspace).expect("member paths");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].path, "crates/member");
    assert!(paths[0].workspace_relative);
}
