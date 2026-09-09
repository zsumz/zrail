//! Exact frozen registry structures and loader, with no surrogate subset.

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub(crate) struct Guardrails {
    pub(crate) schema: u32,
    pub(crate) paths: Paths,
    pub(crate) budgets: Budgets,
    pub(crate) dependencies: Dependencies,
    pub(crate) capabilities: Vec<Capability>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Paths {
    pub(crate) rust_roots: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) struct Budgets {
    pub(crate) facade: usize,
    pub(crate) production: usize,
    pub(crate) test: usize,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Dependencies {
    pub(crate) banned: Vec<String>,
    pub(crate) core_allowed: Vec<String>,
    pub(crate) driver_allowed: Vec<String>,
    pub(crate) probe_allowed: Vec<String>,
    pub(crate) transport_allowed: Vec<String>,
    pub(crate) kafka_wire_version: String,
    pub(crate) kafka_wire_checksum: String,
    pub(crate) kafka_wire_core_version: String,
    pub(crate) kafka_wire_core_checksum: String,
    pub(crate) bornera_version: String,
    pub(crate) bornera_checksum: String,
    pub(crate) bornera_core_version: String,
    pub(crate) bornera_core_checksum: String,
    pub(crate) bornera_rustls_version: String,
    pub(crate) bornera_rustls_checksum: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Capability {
    pub(crate) root: String,
    pub(crate) forbidden: Vec<String>,
}

pub(crate) fn load_guardrails(root: &Path) -> Guardrails {
    let source = read(&root.join("guardrails.toml"));
    let config = toml::from_str::<Guardrails>(&source)
        .unwrap_or_else(|error| panic!("parse guardrails.toml: {error}"));
    assert_eq!(config.schema, 1, "unsupported guardrails.toml schema");
    config
}

pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
