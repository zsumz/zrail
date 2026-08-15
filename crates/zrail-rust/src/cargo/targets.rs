//! Cargo target roots derived from explicit tables and standard auto-discovery.

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

use toml::Value;

use super::{
    model::{CargoTarget, CargoTargetKind},
    target_discovery::{auto_discovery_default, auto_enabled, discover_directory},
};

pub(super) fn collect_target_roots(
    manifest: &Value,
    package_directory: &Path,
    workspace_edition: Option<&str>,
) -> Result<Vec<CargoTarget>, String> {
    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| "Cargo package must contain a [package] table".to_owned())?;
    let mut roots = BTreeSet::new();
    let auto_default = auto_discovery_default(manifest, package, workspace_edition)?;
    collect_library(
        manifest,
        package,
        package_directory,
        auto_default,
        &mut roots,
    )?;
    collect_kind(
        manifest,
        package,
        package_directory,
        "bin",
        "src/bin",
        auto_default,
        CargoTargetKind::Binary,
        &mut roots,
    )?;
    collect_kind(
        manifest,
        package,
        package_directory,
        "example",
        "examples",
        auto_default,
        CargoTargetKind::Example,
        &mut roots,
    )?;
    collect_kind(
        manifest,
        package,
        package_directory,
        "test",
        "tests",
        auto_default,
        CargoTargetKind::Test,
        &mut roots,
    )?;
    collect_kind(
        manifest,
        package,
        package_directory,
        "bench",
        "benches",
        auto_default,
        CargoTargetKind::Benchmark,
        &mut roots,
    )?;
    collect_build_script(package, package_directory, &mut roots)?;
    Ok(roots.into_iter().collect())
}

fn collect_library(
    manifest: &Value,
    package: &toml::map::Map<String, Value>,
    directory: &Path,
    auto_default: bool,
    roots: &mut BTreeSet<CargoTarget>,
) -> Result<(), String> {
    if let Some(library) = manifest.get("lib") {
        let table = library
            .as_table()
            .ok_or_else(|| "Cargo [lib] target must be a table".to_owned())?;
        roots.insert(CargoTarget {
            path: target_path(table, "src/lib.rs")?,
            kind: CargoTargetKind::Library,
        });
    } else if auto_enabled(package, "autolib", auto_default)?
        && directory.join("src/lib.rs").is_file()
    {
        roots.insert(CargoTarget {
            path: "src/lib.rs".into(),
            kind: CargoTargetKind::Library,
        });
    }
    Ok(())
}

fn collect_kind(
    manifest: &Value,
    package: &toml::map::Map<String, Value>,
    directory: &Path,
    kind: &str,
    auto_directory: &str,
    auto_default: bool,
    target_kind: CargoTargetKind,
    roots: &mut BTreeSet<CargoTarget>,
) -> Result<(), String> {
    let mut named = BTreeMap::new();
    let mut explicit = BTreeSet::new();
    if let Some(targets) = manifest.get(kind) {
        let targets = targets
            .as_array()
            .ok_or_else(|| format!("Cargo [[{kind}]] targets must be an array of tables"))?;
        for target in targets {
            let table = target
                .as_table()
                .ok_or_else(|| format!("Cargo [[{kind}]] target must be a table"))?;
            let name = required_string(table, "name", &format!("Cargo [[{kind}]] target"))?;
            let path = explicit_target_path(table, kind, auto_directory, directory)?;
            if named.insert(name.clone(), path).is_some() {
                return Err(format!(
                    "Cargo [[{kind}]] target name {name:?} is duplicated"
                ));
            }
            explicit.insert(name);
        }
    }
    let auto_key = format!("auto{kind}s");
    if auto_enabled(package, &auto_key, auto_default)? {
        if kind == "bin" && directory.join("src/main.rs").is_file() {
            let name = required_string(package, "name", "Cargo package")?;
            add_auto_target(&mut named, &explicit, kind, name, "src/main.rs".into())?;
        }
        for (name, path) in discover_directory(directory, auto_directory)? {
            add_auto_target(&mut named, &explicit, kind, name, path)?;
        }
    }
    roots.extend(named.into_values().map(|path| CargoTarget {
        path,
        kind: target_kind,
    }));
    Ok(())
}

fn add_auto_target(
    targets: &mut BTreeMap<String, String>,
    explicit: &BTreeSet<String>,
    kind: &str,
    name: String,
    path: String,
) -> Result<(), String> {
    if explicit.contains(&name) {
        return Ok(());
    }
    match targets.entry(name) {
        Entry::Vacant(entry) => {
            entry.insert(path);
        }
        Entry::Occupied(entry) => {
            return Err(format!(
                "Cargo auto-discovered {kind} target {:?} is ambiguous between {:?} and {path:?}",
                entry.key(),
                entry.get()
            ));
        }
    }
    Ok(())
}

fn explicit_target_path(
    table: &toml::map::Map<String, Value>,
    kind: &str,
    target_directory: &str,
    package_directory: &Path,
) -> Result<String, String> {
    if let Some(path) = optional_string(table, "path")? {
        return Ok(path);
    }
    let name = required_string(table, "name", &format!("Cargo [[{kind}]] target"))?;
    let direct = format!("{target_directory}/{name}.rs");
    let nested = format!("{target_directory}/{name}/main.rs");
    match (
        package_directory.join(&direct).is_file(),
        package_directory.join(&nested).is_file(),
    ) {
        (false, true) => Ok(nested),
        (true, true) => Err(format!(
            "Cargo [[{kind}]] target {name:?} has ambiguous inferred paths"
        )),
        (_, false) => Ok(direct),
    }
}

fn target_path(table: &toml::map::Map<String, Value>, default: &str) -> Result<String, String> {
    optional_string(table, "path")?.map_or_else(|| Ok(default.into()), Ok)
}

fn collect_build_script(
    package: &toml::map::Map<String, Value>,
    directory: &Path,
    roots: &mut BTreeSet<CargoTarget>,
) -> Result<(), String> {
    match package.get("build") {
        Some(Value::Boolean(false)) => {}
        Some(Value::Boolean(true)) => {
            roots.insert(CargoTarget {
                path: "build.rs".into(),
                kind: CargoTargetKind::BuildScript,
            });
        }
        Some(Value::String(path)) => {
            roots.insert(CargoTarget {
                path: path.clone(),
                kind: CargoTargetKind::BuildScript,
            });
        }
        Some(_) => return Err("package.build must be a path or false".into()),
        None if directory.join("build.rs").is_file() => {
            roots.insert(CargoTarget {
                path: "build.rs".into(),
                kind: CargoTargetKind::BuildScript,
            });
        }
        None => {}
    }
    Ok(())
}

fn required_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<String, String> {
    optional_string(table, key)?.ok_or_else(|| format!("{label} requires {key}"))
}

fn optional_string(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    table.get(key).map_or(Ok(None), |value| {
        value
            .as_str()
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| format!("Cargo target {key} must be a string"))
    })
}

#[cfg(test)]
#[path = "targets_test.rs"]
mod targets_test;
