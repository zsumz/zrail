//! Explicit Cargo target paths and build-script roots.

use std::{collections::BTreeSet, path::Path};

use toml::Value;

use super::{
    model::{CargoTarget, CargoTargetKind},
    target_fields::{optional_string, required_string},
};

pub(super) fn explicit_target_path(
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

pub(super) fn collect_build_script(
    package: &toml::map::Map<String, Value>,
    directory: &Path,
    roots: &mut BTreeSet<CargoTarget>,
) -> Result<(), String> {
    match package.get("build") {
        Some(Value::Boolean(false)) => {}
        Some(Value::Boolean(true)) => insert_build_script(roots, "build.rs"),
        Some(Value::String(path)) => insert_build_script(roots, path),
        Some(_) => return Err("package.build must be a path or false".into()),
        None if directory.join("build.rs").is_file() => insert_build_script(roots, "build.rs"),
        None => {}
    }
    Ok(())
}

fn insert_build_script(roots: &mut BTreeSet<CargoTarget>, path: &str) {
    roots.insert(CargoTarget {
        name: "build-script-build".into(),
        path: path.into(),
        kind: CargoTargetKind::BuildScript,
        required_features: Vec::new(),
    });
}
