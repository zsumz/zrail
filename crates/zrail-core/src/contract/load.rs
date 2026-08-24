//! Deterministic loading of `zrail.toml` and repository-local fragments.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    input::read_text,
    path::{glob_matches, repository_file, repository_relative},
};

/// Maximum combined UTF-8 bytes accepted across a contract and its imports.
pub const MAX_CONTRACT_BYTES: usize = 16 * 1024 * 1024;
/// Maximum number of distinct contract files accepted in one import graph.
pub const MAX_CONTRACT_FILES: usize = 256;
const MAX_IMPORT_DEPTH: usize = 64;
/// Maximum combined import directives accepted across a contract graph.
pub const MAX_IMPORT_DIRECTIVES: usize = 256;

use super::{discover, hash::contract_sha256, merge::MergeState, validate::validate_contract};

mod bundle;
#[path = "load/entry.rs"]
mod entry;
#[path = "load/error.rs"]
mod error;
mod file;
pub use bundle::{ContractBundle, ContractSource};
pub use entry::load_contract_with_entry;
pub use error::ContractError;
pub(super) use file::ContractFile;

/// Loads, merges, and validates a repository-bounded architecture contract.
/// `root` is canonicalized; `config` and every import must be regular, non-symlink files inside it.
/// Imports are deterministic. Inaccessible files, escapes, aliases, import-graph errors, malformed
/// TOML, unknown keys, merge or validation failures, and safety-limit violations return
/// [`ContractError`]; no partial contract is returned.
pub fn load_contract(root: &Path, config: &Path) -> Result<ContractBundle, ContractError> {
    load_contract_entry(root, config, None)
}

pub(super) fn load_contract_entry(
    root: &Path,
    config: &Path,
    entry: Option<&str>,
) -> Result<ContractBundle, ContractError> {
    let root = fs::canonicalize(root).map_err(|error| {
        ContractError::one(format!("open repository {}: {error}", root.display()))
    })?;
    let config = repository_file(&root, config).map_err(ContractError::one)?;
    if let Some(content) = entry {
        entry::validate(&config, content)?;
    }
    let mut loader = Loader::new(root);
    loader.load(&config, entry)?;
    let contract = loader.state.finish()?;
    validate_contract(&contract)?;
    for source in &loader.sources {
        if contract
            .repository
            .exclude
            .iter()
            .any(|pattern| glob_matches(pattern, &source.path))
        {
            return Err(ContractError::one(format!(
                "excluded repository file cannot provide architecture policy: {}",
                source.path
            )));
        }
    }
    loader
        .sources
        .sort_by(|left, right| left.path.cmp(&right.path));
    let sha256 = contract_sha256(&loader.sources);
    Ok(ContractBundle {
        contract,
        sources: loader.sources,
        sha256,
    })
}

#[derive(Debug)]
struct Loader {
    root: PathBuf,
    state: MergeState,
    sources: Vec<ContractSource>,
    visited: BTreeSet<PathBuf>,
    stack: Vec<PathBuf>,
    bytes: usize,
    imports: usize,
    toml_files: BTreeMap<String, Vec<PathBuf>>,
    discovered_entries: usize,
    schema: Option<u64>,
}

impl Loader {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            state: MergeState::default(),
            sources: Vec::new(),
            visited: BTreeSet::new(),
            stack: Vec::new(),
            bytes: 0,
            imports: 0,
            toml_files: BTreeMap::new(),
            discovered_entries: 0,
            schema: None,
        }
    }

    fn load(&mut self, path: &Path, entry: Option<&str>) -> Result<(), ContractError> {
        if self.stack.len() == MAX_IMPORT_DEPTH {
            return Err(ContractError::one(format!(
                "contract imports exceed the {MAX_IMPORT_DEPTH}-level safety limit"
            )));
        }
        if self.visited.len() == MAX_CONTRACT_FILES {
            return Err(ContractError::one(format!(
                "contract imports exceed the {MAX_CONTRACT_FILES}-file safety limit"
            )));
        }
        let canonical = fs::canonicalize(path)
            .map_err(|error| ContractError::one(format!("read {}: {error}", path.display())))?;
        if !canonical.starts_with(&self.root) {
            return Err(ContractError::one(format!(
                "contract import escapes repository: {}",
                path.display()
            )));
        }
        let origin = repository_relative(&self.root, &canonical).map_err(ContractError::one)?;
        if self.stack.contains(&canonical) {
            return Err(ContractError::one(format!(
                "contract import cycle reaches {origin}"
            )));
        }
        if !self.visited.insert(canonical.clone()) {
            return Err(ContractError::one(format!(
                "duplicate contract import: {origin}"
            )));
        }
        self.stack.push(canonical.clone());
        let content = entry
            .map(str::to_owned)
            .map_or_else(|| read_text(path).map_err(ContractError::one), Ok)?;
        self.bytes = self
            .bytes
            .checked_add(content.len())
            .ok_or_else(|| ContractError::one("contract source byte count overflowed"))?;
        if self.bytes > MAX_CONTRACT_BYTES {
            return Err(ContractError::one(format!(
                "contract imports exceed the {MAX_CONTRACT_BYTES}-byte safety limit"
            )));
        }
        let file = toml::from_str::<ContractFile>(&content).map_err(|error| {
            ContractError::one(format!("parse {}: {error}", canonical.display()))
        })?;
        if self.schema.is_none() {
            self.schema = file.schema;
        }
        self.imports = self
            .imports
            .checked_add(file.imports.len())
            .ok_or_else(|| ContractError::one("contract import count overflowed"))?;
        if self.imports > MAX_IMPORT_DIRECTIVES {
            return Err(ContractError::one(format!(
                "contract imports exceed the {MAX_IMPORT_DIRECTIVES}-directive safety limit"
            )));
        }
        let imports = self.expand_imports(&file.imports)?;
        self.sources.push(ContractSource {
            path: origin.clone(),
            content,
        });
        self.state.merge(file, &origin)?;
        for import in imports {
            self.load(&import, None)?;
        }
        self.stack.pop();
        Ok(())
    }

    fn expand_imports(&mut self, imports: &[String]) -> Result<Vec<PathBuf>, ContractError> {
        let mut expanded = Vec::new();
        for import in imports {
            let import = discover::normalize_import(import)?;
            if self.schema == Some(2) && discover::has_wildcard(&import) {
                return Err(ContractError::one(format!(
                    "schema-2 contract imports must be exact paths, not patterns: {import:?}"
                )));
            }
            if !discover::has_wildcard(&import) {
                expanded.push(self.root.join(import));
                continue;
            }
            let prefix = discover::fixed_prefix(&import);
            if !self.toml_files.contains_key(&prefix) {
                let files =
                    discover::toml_files(&self.root, &prefix, &mut self.discovered_entries)?;
                self.toml_files.insert(prefix.clone(), files);
            }
            let candidates = &self.toml_files[&prefix];
            let mut matches = Vec::new();
            for path in candidates {
                let relative = repository_relative(&self.root, path).map_err(ContractError::one)?;
                if glob_matches(&import, &relative) {
                    matches.push(path.clone());
                }
            }
            if matches.is_empty() {
                return Err(ContractError::one(format!(
                    "contract import {import:?} matched no files"
                )));
            }
            expanded.extend(matches);
        }
        expanded.sort();
        Ok(expanded)
    }
}

#[cfg(test)]
#[path = "load_test.rs"]
mod load_test;
