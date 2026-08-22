//! Active workspace manifests are selected before full package resolution.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Path, PathBuf},
};

use toml::Value;

use crate::inventory::RepositoryInventory;

use super::{
    dependencies::collect_dependencies,
    dependency_spec::WorkspaceDependencies,
    parse::{CargoModelError, package_name, read_manifest_counted},
    workspace::{
        excluded_member, expand_members, normalized_directory, workspace_excludes,
        workspace_members,
    },
};

pub(super) struct WorkspacePlan {
    pub(super) member_patterns: Vec<String>,
    pub(super) exclude_patterns: Vec<String>,
    pub(super) selected_manifests: BTreeSet<PathBuf>,
    pub(super) observed_extras: Vec<String>,
    pub(super) excluded_boundaries: Vec<String>,
    pub(super) nested_workspace_boundaries: Vec<String>,
    values: BTreeMap<PathBuf, Value>,
}

impl WorkspacePlan {
    pub(super) fn value(&self, manifest: &Path) -> Result<&Value, CargoModelError> {
        self.values
            .get(manifest)
            .ok_or_else(|| CargoModelError("selected workspace manifest was not loaded".into()))
    }
}

pub(super) fn build(
    inventory: &RepositoryInventory,
    root_manifest: &Path,
    root: Value,
    workspace_dependencies: &WorkspaceDependencies,
    root_package: bool,
    manifest_bytes: &mut usize,
) -> Result<WorkspacePlan, CargoModelError> {
    let member_patterns = workspace_members(&root)?;
    let exclude_patterns = workspace_excludes(&root)?;
    let candidates = candidate_manifests(inventory, root_manifest)?;
    let observed = candidates
        .keys()
        .filter(|directory| !excluded_member(directory, &exclude_patterns))
        .cloned()
        .collect::<Vec<_>>();
    let explicit = expand_members(&member_patterns, &observed, root_package)?;
    let mut selected = explicit.iter().cloned().collect::<BTreeSet<_>>();
    selected.insert(".".into());
    let mut origins = explicit
        .into_iter()
        .map(|directory| (directory, None))
        .collect::<BTreeMap<_, _>>();
    origins.entry(".".into()).or_insert(None);
    let mut queue = selected.iter().cloned().collect::<VecDeque<_>>();
    let mut values = BTreeMap::from([(root_manifest.to_path_buf(), root)]);
    while let Some(directory) = queue.pop_front() {
        reject_nested_ancestor(
            &directory,
            origins.get(&directory).and_then(Option::as_deref),
            &candidates,
            &mut values,
            manifest_bytes,
        )?;
        let manifest = candidates.get(&directory).ok_or_else(|| {
            CargoModelError(format!(
                "selected Cargo package directory {directory:?} has no manifest"
            ))
        })?;
        let value = load(manifest, &mut values, manifest_bytes)?;
        if directory != "." && value.get("workspace").is_some() {
            return Err(boundary_error(
                origins.get(&directory).and_then(Option::as_deref),
                &directory,
            ));
        }
        if directory != "." && package_name(value)?.is_none() {
            return Err(CargoModelError(format!(
                "selected Cargo manifest {} contains no [package] table",
                manifest.display()
            )));
        }
        let dependencies = collect_dependencies(value, workspace_dependencies, &directory)
            .map_err(CargoModelError)?;
        for target in dependencies
            .iter()
            .filter_map(|dependency| dependency.repository_path())
        {
            if !candidates.contains_key(target) {
                return Err(CargoModelError(format!(
                    "path dependency from {directory:?} names missing package directory {target:?}"
                )));
            }
            if selected.insert(target.into()) {
                origins.insert(target.into(), Some(directory.clone()));
                queue.push_back(target.into());
            }
        }
    }
    let selected_manifests = selected
        .iter()
        .filter_map(|directory| candidates.get(directory).cloned())
        .collect::<BTreeSet<_>>();
    let unselected = classify_unselected(
        &candidates,
        &selected,
        &exclude_patterns,
        &mut values,
        manifest_bytes,
    )?;
    Ok(WorkspacePlan {
        member_patterns,
        exclude_patterns,
        selected_manifests,
        observed_extras: unselected.observed,
        excluded_boundaries: unselected.excluded,
        nested_workspace_boundaries: unselected.nested,
        values,
    })
}

struct UnselectedScopes {
    observed: Vec<String>,
    excluded: Vec<String>,
    nested: Vec<String>,
}

fn classify_unselected(
    candidates: &BTreeMap<String, PathBuf>,
    selected: &BTreeSet<String>,
    exclude_patterns: &[String],
    values: &mut BTreeMap<PathBuf, Value>,
    manifest_bytes: &mut usize,
) -> Result<UnselectedScopes, CargoModelError> {
    let mut directories = candidates.keys().cloned().collect::<Vec<_>>();
    directories.sort_by_key(|directory| (directory.matches('/').count(), directory.clone()));
    let mut scopes = UnselectedScopes {
        observed: Vec::new(),
        excluded: Vec::new(),
        nested: Vec::new(),
    };
    for directory in directories {
        if selected.contains(&directory) {
            continue;
        }
        if excluded_member(&directory, exclude_patterns) {
            scopes.excluded.push(directory);
            continue;
        }
        if beneath_boundary(&directory, &scopes.excluded)
            || beneath_boundary(&directory, &scopes.nested)
        {
            continue;
        }
        let manifest = &candidates[&directory];
        let value = load(manifest, values, manifest_bytes)?;
        if value.get("workspace").is_some() {
            scopes.nested.push(directory);
        } else if package_name(value)?.is_some() {
            scopes.observed.push(directory);
        } else {
            return Err(CargoModelError(format!(
                "unlisted Cargo manifest {} contains no [package] or [workspace] table",
                manifest.display()
            )));
        }
    }
    Ok(scopes)
}

fn beneath_boundary(directory: &str, boundaries: &[String]) -> bool {
    boundaries
        .iter()
        .any(|boundary| directory == boundary || directory.starts_with(&format!("{boundary}/")))
}

fn candidate_manifests(
    inventory: &RepositoryInventory,
    root_manifest: &Path,
) -> Result<BTreeMap<String, PathBuf>, CargoModelError> {
    let mut candidates = BTreeMap::new();
    candidates.insert(".".into(), root_manifest.to_path_buf());
    for manifest in &inventory.manifest_paths {
        candidates.insert(
            normalized_directory(&inventory.root, manifest)?,
            manifest.clone(),
        );
    }
    Ok(candidates)
}

fn reject_nested_ancestor(
    directory: &str,
    origin: Option<&str>,
    candidates: &BTreeMap<String, PathBuf>,
    values: &mut BTreeMap<PathBuf, Value>,
    manifest_bytes: &mut usize,
) -> Result<(), CargoModelError> {
    let mut ancestor = directory.to_owned();
    while let Some((parent, _)) = ancestor.rsplit_once('/') {
        ancestor = parent.into();
        let Some(manifest) = candidates.get(&ancestor) else {
            continue;
        };
        if load(manifest, values, manifest_bytes)?
            .get("workspace")
            .is_some()
        {
            return Err(boundary_error(origin, &ancestor));
        }
    }
    Ok(())
}

fn load<'a>(
    manifest: &Path,
    values: &'a mut BTreeMap<PathBuf, Value>,
    manifest_bytes: &mut usize,
) -> Result<&'a Value, CargoModelError> {
    if !values.contains_key(manifest) {
        let value = read_manifest_counted(manifest, manifest_bytes)?;
        values.insert(manifest.to_path_buf(), value);
    }
    values
        .get(manifest)
        .ok_or_else(|| CargoModelError("Cargo manifest cache lost a loaded value".into()))
}

fn boundary_error(origin: Option<&str>, nested: &str) -> CargoModelError {
    let edge = origin.map_or_else(
        || "workspace member".to_owned(),
        |origin| format!("path dependency from {origin:?}"),
    );
    CargoModelError(format!(
        "{edge} crosses from workspace \".\" into nested workspace {nested:?}; multi-workspace dependency resolution is not yet supported"
    ))
}
