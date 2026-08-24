//! Bounded Cargo.lock parsing resolves every textual dependency reference exactly.

use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path};

use toml::Value;
use zrail_core::read_text_with_limit;

use crate::cargo::CargoModelError;

const MAX_LOCK_BYTES: usize = 16 * 1024 * 1024;
const MAX_LOCK_PACKAGES: usize = 100_000;
const MAX_LOCK_EDGES: usize = 1_000_000;

mod fields;

use fields::{
    label, optional_string, package_id, resolve_reference, string_array, validate_provenance,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RawPackageId {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) source: Option<String>,
}

#[derive(Debug)]
pub(super) struct RawPackage {
    pub(super) checksum: Option<String>,
    pub(super) dependencies: Vec<RawPackageId>,
}

pub(super) type RawGraph = BTreeMap<RawPackageId, RawPackage>;

pub(super) fn load(root: &Path) -> Result<Option<(RawGraph, String)>, CargoModelError> {
    let path = root.join("Cargo.lock");
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(CargoModelError(format!(
                "inspect {}: {error}",
                path.display()
            )));
        }
        Ok(_) => {}
    }
    let source = read_text_with_limit(&path, MAX_LOCK_BYTES).map_err(CargoModelError)?;
    let value = source
        .parse::<Value>()
        .map_err(|error| CargoModelError(format!("parse {}: {error}", path.display())))?;
    parse(&value)
        .map(|graph| (graph, zrail_core::sha256_hex(source.as_bytes())))
        .map(Some)
}

fn parse(value: &Value) -> Result<RawGraph, CargoModelError> {
    let version = value
        .get("version")
        .and_then(Value::as_integer)
        .ok_or_else(|| CargoModelError("Cargo.lock requires integer version".into()))?;
    if !matches!(version, 3 | 4) {
        return Err(CargoModelError(format!(
            "Cargo.lock version {version} is unsupported; expected 3 or 4"
        )));
    }
    let packages = value
        .get("package")
        .and_then(Value::as_array)
        .ok_or_else(|| CargoModelError("Cargo.lock requires [[package]] entries".into()))?;
    if packages.len() > MAX_LOCK_PACKAGES {
        return Err(CargoModelError(format!(
            "Cargo.lock exceeds the {MAX_LOCK_PACKAGES}-package safety limit"
        )));
    }
    let mut drafts = BTreeMap::new();
    for package in packages {
        let table = package
            .as_table()
            .ok_or_else(|| CargoModelError("Cargo.lock package must be a table".into()))?;
        let id = package_id(table)?;
        let checksum = optional_string(table, "checksum")?;
        validate_provenance(&id, checksum.as_deref())?;
        let dependencies = string_array(table.get("dependencies"), "package.dependencies")?;
        if drafts
            .insert(id.clone(), (checksum, dependencies))
            .is_some()
        {
            return Err(CargoModelError(format!(
                "Cargo.lock contains duplicate package {}",
                label(&id)
            )));
        }
    }
    let mut edges = 0_usize;
    let mut graph = BTreeMap::new();
    for (id, (checksum, references)) in &drafts {
        edges = edges
            .checked_add(references.len())
            .ok_or_else(|| CargoModelError("Cargo.lock dependency count overflowed".into()))?;
        if edges > MAX_LOCK_EDGES {
            return Err(CargoModelError(format!(
                "Cargo.lock exceeds the {MAX_LOCK_EDGES}-edge safety limit"
            )));
        }
        let mut dependencies = references
            .iter()
            .map(|reference| resolve_reference(reference, drafts.keys()))
            .collect::<Result<Vec<_>, _>>()?;
        dependencies.sort();
        dependencies.dedup();
        graph.insert(
            id.clone(),
            RawPackage {
                checksum: checksum.clone(),
                dependencies,
            },
        );
    }
    Ok(graph)
}
