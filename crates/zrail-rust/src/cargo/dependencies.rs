//! Direct dependency extraction across kinds and target selectors.

use std::collections::BTreeSet;

use toml::Value;

use super::{
    dependency_spec::{self, DependencySpec, WorkspaceDependencies},
    model::{CrateRootAuthority, Dependency, DependencyKind, rust_crate_root},
};

pub(super) fn workspace_dependencies(value: &Value) -> Result<WorkspaceDependencies, String> {
    let Some(workspace) = value.get("workspace") else {
        return Ok(WorkspaceDependencies::new());
    };
    let workspace = workspace
        .as_table()
        .ok_or_else(|| "Cargo [workspace] must be a table".to_owned())?;
    let Some(dependencies) = workspace.get("dependencies") else {
        return Ok(WorkspaceDependencies::new());
    };
    let dependencies = dependencies
        .as_table()
        .ok_or_else(|| "Cargo [workspace.dependencies] must be a table".to_owned())?;
    dependencies
        .iter()
        .map(|(alias, value)| {
            if value
                .as_table()
                .is_some_and(|table| table.contains_key("optional"))
            {
                return Err(format!(
                    "workspace dependency {alias:?}: optional is only valid at the inheriting package"
                ));
            }
            dependency_spec::parse(alias, value, ".", None)
                .map(|spec| (alias.clone(), spec))
                .map_err(|error| format!("workspace dependency {alias:?}: {error}"))
        })
        .collect()
}

pub(super) fn collect_dependencies(
    value: &Value,
    workspace: &WorkspaceDependencies,
    package_directory: &str,
) -> Result<Vec<Dependency>, String> {
    let mut result = BTreeSet::new();
    collect_tables(value, workspace, package_directory, None, &mut result)?;
    if let Some(targets) = value.get("target") {
        let targets = targets
            .as_table()
            .ok_or_else(|| "Cargo [target] must be a table".to_owned())?;
        for (selector, target) in targets {
            if selector.trim().is_empty() {
                return Err("Cargo target selector may not be empty".into());
            }
            if !target.is_table() {
                return Err(format!(
                    "Cargo target selector {selector:?} must contain a table"
                ));
            }
            collect_tables(
                target,
                workspace,
                package_directory,
                Some(selector),
                &mut result,
            )?;
        }
    }
    Ok(result.into_iter().collect())
}

fn collect_tables(
    value: &Value,
    workspace: &WorkspaceDependencies,
    package_directory: &str,
    target: Option<&str>,
    result: &mut BTreeSet<Dependency>,
) -> Result<(), String> {
    let context = TableContext {
        workspace,
        package_directory,
        target,
    };
    for (key, kind) in [
        ("dependencies", DependencyKind::Normal),
        ("dev-dependencies", DependencyKind::Development),
        ("build-dependencies", DependencyKind::Build),
    ] {
        collect_dependency_table(value, key, kind, &context, result)?;
    }
    Ok(())
}

fn collect_dependency_table(
    value: &Value,
    key: &str,
    kind: DependencyKind,
    context: &TableContext<'_>,
    result: &mut BTreeSet<Dependency>,
) -> Result<(), String> {
    let Some(table) = value.get(key) else {
        return Ok(());
    };
    let table = table
        .as_table()
        .ok_or_else(|| format!("Cargo [{key}] must be a table"))?;
    for (alias, value) in table {
        let spec = dependency_spec::parse(
            alias,
            value,
            context.package_directory,
            Some(context.workspace),
        )
        .map_err(|error| format!("dependency {alias:?}: {error}"))?;
        result.insert(dependency(alias, kind, context.target, spec));
    }
    Ok(())
}

struct TableContext<'a> {
    workspace: &'a WorkspaceDependencies,
    package_directory: &'a str,
    target: Option<&'a str>,
}

fn dependency(
    alias: &str,
    kind: DependencyKind,
    target: Option<&str>,
    spec: DependencySpec,
) -> Dependency {
    Dependency {
        alias: alias.into(),
        name: spec.name,
        explicit_package: spec.explicit_package,
        crate_root: rust_crate_root(alias),
        crate_root_authority: if spec.explicit_package {
            CrateRootAuthority::DeclaredAlias
        } else {
            CrateRootAuthority::Unresolved
        },
        kind,
        target: target.map(str::to_owned),
        optional: spec.optional,
        default_features: spec.default_features,
        features: spec.features,
        source: spec.source,
    }
}

#[cfg(test)]
#[path = "dependencies_test.rs"]
mod dependencies_test;
