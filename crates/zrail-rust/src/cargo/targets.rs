//! Cargo target roots derived from explicit tables and standard auto-discovery.

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::Path,
};

use toml::Value;

use super::{
    model::{CargoTarget, CargoTargetKind},
    target_discovery::{auto_discovery_default, auto_enabled, discover_directory},
    target_explicit::{collect_build_script, explicit_target_path},
    target_fields::{optional_string, required_string, string_array, target_path},
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
    let package_name = required_string(package, "name", "Cargo package")?;
    let mut roots = BTreeSet::new();
    let auto_default = auto_discovery_default(manifest, package, workspace_edition)?;
    collect_library(
        manifest,
        package,
        &package_name,
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
    package_name: &str,
    directory: &Path,
    auto_default: bool,
    roots: &mut BTreeSet<CargoTarget>,
) -> Result<(), String> {
    if let Some(library) = manifest.get("lib") {
        let table = library
            .as_table()
            .ok_or_else(|| "Cargo [lib] target must be a table".to_owned())?;
        let name =
            optional_string(table, "name")?.unwrap_or_else(|| super::rust_crate_root(package_name));
        validate_library_name(&name)?;
        roots.insert(CargoTarget {
            name,
            path: target_path(table, "src/lib.rs")?,
            kind: CargoTargetKind::Library,
            required_features: Vec::new(),
        });
    } else if auto_enabled(package, "autolib", auto_default)?
        && directory.join("src/lib.rs").is_file()
    {
        let name = super::rust_crate_root(package_name);
        validate_library_name(&name)?;
        roots.insert(CargoTarget {
            name,
            path: "src/lib.rs".into(),
            kind: CargoTargetKind::Library,
            required_features: Vec::new(),
        });
    }
    Ok(())
}

fn validate_library_name(name: &str) -> Result<(), String> {
    if matches!(name, "_" | "Self" | "crate" | "self" | "super") {
        return Err(format!(
            "Cargo [lib] name {name:?} must be one usable Rust crate identifier"
        ));
    }
    let mut bytes = name.bytes();
    let valid = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    if valid {
        Ok(())
    } else {
        Err(format!(
            "Cargo [lib] name {name:?} must be one usable Rust crate identifier"
        ))
    }
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
            let required_features = string_array(table, "required-features")?;
            if named
                .insert(name.clone(), (path, required_features))
                .is_some()
            {
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
    roots.extend(
        named
            .into_iter()
            .map(|(name, (path, required_features))| CargoTarget {
                name,
                path,
                kind: target_kind,
                required_features,
            }),
    );
    Ok(())
}

fn add_auto_target(
    targets: &mut BTreeMap<String, (String, Vec<String>)>,
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
            entry.insert((path, Vec::new()));
        }
        Entry::Occupied(entry) => {
            return Err(format!(
                "Cargo auto-discovered {kind} target {:?} is ambiguous between {:?} and {path:?}",
                entry.key(),
                entry.get().0
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "targets_test.rs"]
mod targets_test;
