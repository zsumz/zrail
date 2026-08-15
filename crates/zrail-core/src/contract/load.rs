//! Deterministic loading of `zrail.toml` and repository-local fragments.

use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{
    input::read_text,
    path::{glob_matches, repository_file, repository_relative},
};

pub const MAX_CONTRACT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_CONTRACT_FILES: usize = 256;
const MAX_IMPORT_DEPTH: usize = 64;
pub const MAX_IMPORT_DIRECTIVES: usize = 256;

use super::{discover, hash::contract_sha256, merge::MergeState, validate::validate_contract};

mod file;
pub(super) use file::ContractFile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSource {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractBundle {
    pub contract: super::Contract,
    pub sources: Vec<ContractSource>,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractError {
    messages: Vec<String>,
}

impl ContractError {
    pub fn one(message: impl Into<String>) -> Self {
        Self {
            messages: vec![message.into()],
        }
    }

    pub(crate) fn many(messages: Vec<String>) -> Self {
        Self { messages }
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.messages.join("\n"))
    }
}

impl Error for ContractError {}

pub fn load_contract(root: &Path, config: &Path) -> Result<ContractBundle, ContractError> {
    let root = fs::canonicalize(root).map_err(|error| {
        ContractError::one(format!("open repository {}: {error}", root.display()))
    })?;
    let config = repository_file(&root, config).map_err(ContractError::one)?;
    let mut loader = Loader::new(root);
    loader.load(&config)?;
    let contract = loader.state.finish()?;
    validate_contract(&contract)?;
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
    toml_files: Option<Vec<PathBuf>>,
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
            toml_files: None,
        }
    }

    fn load(&mut self, path: &Path) -> Result<(), ContractError> {
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
        let content = read_text(path).map_err(ContractError::one)?;
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
            self.load(&import)?;
        }
        self.stack.pop();
        Ok(())
    }

    fn expand_imports(&mut self, imports: &[String]) -> Result<Vec<PathBuf>, ContractError> {
        let mut expanded = Vec::new();
        for import in imports {
            if !discover::has_wildcard(import) {
                expanded.push(self.root.join(import));
                continue;
            }
            let candidates = if let Some(files) = &self.toml_files {
                files
            } else {
                self.toml_files.insert(discover::toml_files(&self.root)?)
            };
            let mut matches = Vec::new();
            for path in candidates {
                let relative = repository_relative(&self.root, path).map_err(ContractError::one)?;
                if glob_matches(import, &relative) {
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
