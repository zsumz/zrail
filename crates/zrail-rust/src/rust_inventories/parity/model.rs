//! Frozen syntax inputs and explicit scope-only reports cannot stand in for repository certificates.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zrail_core::{RustInventoryRule, sha256_hex};

use crate::{GovernedRustInventory, RustInventoryInput};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    source: Source,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    rust: Rust,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rust {
    inventories: Vec<RustInventoryRule>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project root")
        .to_path_buf()
}

pub(super) fn policy() -> (RustInventoryRule, String) {
    let bytes =
        fs::read(project().join("docs/rc9/policies/kafka-driver.transport-methods.fragment.toml"))
            .expect("reviewed transport policy");
    let mut fragment: Fragment = toml::from_str(std::str::from_utf8(&bytes).expect("UTF-8 policy"))
        .expect("strict typed policy fragment");
    assert_eq!(fragment.source.rust.inventories.len(), 1);
    (
        fragment.source.rust.inventories.remove(0),
        sha256_hex(&bytes),
    )
}

pub(super) fn inputs(root: &Path, roots: &[String]) -> BTreeMap<String, String> {
    let files = crate::rules::legacy_driver_paths(root, roots);
    let sources = files
        .iter()
        .map(|path| {
            (
                super::selection::display_path(root, path),
                super::selection::read(path),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        files.len(),
        sources.len(),
        "frozen roots do not duplicate physical files"
    );
    sources
}

pub(super) fn hashes(sources: &BTreeMap<String, String>) -> Vec<RustInventoryInput> {
    sources
        .iter()
        .map(|(path, source)| RustInventoryInput {
            path: path.clone(),
            sha256: sha256_hex(source.as_bytes()),
            bytes: source.len(),
        })
        .collect()
}

pub(super) fn detector_source() -> String {
    let source =
        fs::read_to_string(project().join(
            "crates/zrail-testkit/tests/fixtures/rc9/transport_methods/transport_authority.rs",
    ))
    .expect("bound original detector source");
    assert_eq!(
        sha256_hex(source.as_bytes()),
        "6586263df79223c6d4e7f147b2d17706f6a4837b55b43c9b981c00ae5c4481ed",
        "parse only the exact frozen detector bytes",
    );
    let syntax = syn::parse_file(&source).expect("exact original detector source");
    let function = syntax
        .items
        .iter()
        .find_map(|item| {
            if let syn::Item::Fn(function) = item
                && function.sig.ident == "inventory_detects_alias_ufcs_and_guarded_renames"
            {
                Some(function)
            } else {
                None
            }
        })
        .expect("original detector test");
    let syn::Stmt::Local(local) = &function.block.stmts[0] else {
        panic!("original source binding")
    };
    let syn::Expr::Lit(expression) = local
        .init
        .as_ref()
        .expect("source initializer")
        .expr
        .as_ref()
    else {
        panic!("literal detector source")
    };
    let syn::Lit::Str(source) = &expression.lit else {
        panic!("raw source string")
    };
    source.value()
}

#[derive(Serialize)]
pub(super) struct Report {
    pub(super) schema: u32,
    pub(super) implementation_commit: String,
    pub(super) implementation_tree: String,
    pub(super) snapshot: serde_json::Value,
    pub(super) policy_sha256: String,
    pub(super) registry_sha256: String,
    pub(super) fixture_origins_sha256: String,
    pub(super) test_binary_sha256: String,
    pub(super) cargo_lock_sha256: String,
    pub(super) rustc_version: String,
    pub(super) all_source_inputs: Vec<RustInventoryInput>,
    pub(super) observation: GovernedRustInventory,
    pub(super) legacy_counts: BTreeMap<String, usize>,
    pub(super) detector_source_sha256: String,
    pub(super) detector_counts: BTreeMap<String, usize>,
    pub(super) fixture_root: String,
    pub(super) fixtures: Vec<Fixture>,
    pub(super) full_repository_qualified: bool,
    pub(super) limitations: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct Fixture {
    pub(super) case: String,
    pub(super) policy_id: String,
    pub(super) changed_inputs: Vec<RustInventoryInput>,
    pub(super) removed_inputs: Vec<String>,
    pub(super) legacy_fixture_parses: BTreeMap<String, bool>,
    pub(super) legacy_counts: Option<BTreeMap<String, usize>>,
    pub(super) native: Option<GovernedRustInventory>,
    pub(super) legacy_accepted: bool,
    pub(super) native_accepted: bool,
    pub(super) diagnostic: Option<String>,
}
