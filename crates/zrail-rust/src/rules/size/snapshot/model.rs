//! Trusted size-only snapshot inputs reuse native physical inventory and budget facts.

use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

use serde::Deserialize;
use zrail_core::{Contract, RatchetContract, SourceContract, load_contract, sha256_hex};

use crate::inventory::{RepositoryInventory, inventory_repository};

#[derive(Clone, Copy, Deserialize)]
pub(super) struct Budget {
    pub(super) target: usize,
    pub(super) soft: usize,
    pub(super) hard: usize,
}

#[derive(Deserialize)]
pub(super) struct BudgetBaseline {
    pub(super) path: String,
    pub(super) lines: usize,
    pub(super) reason: String,
}

#[derive(Deserialize)]
pub(super) struct BudgetAllow {
    pub(super) path: String,
    pub(super) reason: String,
    pub(super) owner: String,
    pub(super) issue: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fragment {
    pub(super) source: SourceContract,
    #[serde(default)]
    pub(super) ratchet: Vec<RatchetContract>,
}

pub(super) struct Snapshot {
    pub(super) root: PathBuf,
    pub(super) contract: Contract,
    pub(super) inventory: RepositoryInventory,
    pub(super) config: toml::Value,
    pub(super) policy_sha256: String,
    pub(super) config_sha256: String,
    pub(super) pin: serde_json::Value,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("project directory")
        .to_path_buf()
}

pub(super) fn fragment(name: &str) -> (Fragment, String) {
    let source =
        fs::read_to_string(project().join(format!("docs/rc9/policies/{name}.sizes.fragment.toml")))
            .expect("read reviewed fragment");
    let fragment = toml::from_str(&format!(
        "[source.rust]\nmodule_docs = 'allow'\nfacades = 'allow'\ntests = 'allow'\n\
         [source.rust.hygiene]\nunsafe = 'allow'\nlint_suppressions = 'allow'\n{source}"
    ))
    .expect("parse strict typed size fragment");
    (fragment, sha256_hex(source.as_bytes()))
}

pub(super) fn git(root: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("trusted snapshot Git inspection");
    assert!(output.status.success(), "{:?}", output.stderr);
    String::from_utf8(output.stdout).expect("UTF-8 Git output")
}

impl Snapshot {
    pub(super) fn load(name: &str) -> Self {
        let roots = std::env::var_os("ZRAIL_RC9_SNAPSHOTS").expect("prefetch frozen snapshots");
        let root = PathBuf::from(roots).join(name);
        let pins: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../zrail-testkit/tests/fixtures/rc9/snapshots.json"
        ))
        .expect("parse snapshot pins");
        let pin = pins["consumers"]
            .as_array()
            .expect("consumer pins")
            .iter()
            .find(|pin| pin["name"] == name)
            .expect("named snapshot")
            .clone();
        let source = fs::read_to_string(root.join("guardrails.toml")).expect("legacy policy");
        let config: toml::Value = toml::from_str(&source).expect("parse frozen policy");
        let (fragment, policy_sha256) = fragment(name);
        let mut contract = load_contract(&project(), std::path::Path::new("zrail.toml"))
            .expect("load native inventory defaults")
            .contract;
        contract.repository.roots = config["paths"]["rust_roots"]
            .clone()
            .try_into()
            .expect("source roots");
        contract.repository.exclude = config["paths"]
            .get("excluded_roots")
            .map(|paths| {
                paths
                    .as_array()
                    .expect("excluded roots")
                    .iter()
                    .map(|path| format!("{}/**", path.as_str().expect("exact excluded root")))
                    .collect()
            })
            .unwrap_or_default();
        contract.source = fragment.source;
        contract.ratchets = fragment.ratchet;
        let inventory = inventory_repository(&root, &contract).expect("native source inventory");
        let snapshot = Self {
            root,
            contract,
            inventory,
            config,
            policy_sha256,
            config_sha256: sha256_hex(source.as_bytes()),
            pin,
        };
        snapshot.verify();
        snapshot
    }

    pub(super) fn verify(&self) {
        for (expression, field) in [("HEAD", "commit"), ("HEAD^{tree}", "tree")] {
            assert_eq!(
                git(&self.root, &["rev-parse", expression]).trim(),
                self.pin[field]
            );
        }
        assert!(
            git(
                &self.root,
                &["status", "--porcelain=v1", "--untracked-files=all"]
            )
            .is_empty()
        );
    }

    pub(super) fn budgets(&self) -> BTreeMap<String, Budget> {
        ["facade", "implementation", "test", "auxiliary"]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    self.config["budgets"][name]
                        .clone()
                        .try_into()
                        .expect("budget"),
                )
            })
            .collect()
    }
}
