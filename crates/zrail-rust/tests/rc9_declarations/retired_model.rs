//! Test-owned physical fixtures invoke the unchanged stock file and Rust inventory analyzers.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use zrail_core::{RepositoryFileRule, RustInventoryRule, sha256_hex};

pub(super) const MODULES: &str = "src/reactor/mod.rs";
pub(super) const BACKEND: &str = "src/reactor/backend.rs";
pub(super) const CONSTRUCTION: &str = "src/reactor/host/construction.rs";
pub(super) const NAMES: [&str; 7] = [
    "broker_set",
    "plaintext",
    "poller",
    "resource",
    "tcp",
    "timer",
    "tls",
];
pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.retired.fragment.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fragment {
    pub(super) repository: Repository,
    pub(super) source: Source,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Repository {
    pub(super) files: Vec<RepositoryFileRule>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub(super) rust: Rust,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Rust {
    pub(super) inventories: Vec<RustInventoryRule>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project")
        .to_owned()
}

pub(super) fn policies() -> Fragment {
    let policy: Fragment =
        toml::from_str(&fs::read_to_string(project().join(POLICY)).expect("policy"))
            .expect("strict fragment");
    assert_eq!(policy.repository.files.len(), 10);
    assert_eq!(policy.source.rust.inventories.len(), 2);
    policy
}

pub(super) struct Fixture {
    pub(super) root: PathBuf,
    pub(super) inputs: BTreeMap<String, String>,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self::at(
            std::env::temp_dir()
                .canonicalize()
                .expect("canonical temporary directory")
                .join(format!(
                    "zrail-retired-{}-{:?}",
                    std::process::id(),
                    std::thread::current().id()
                )),
            BTreeMap::from([
                (MODULES.into(), "//! Reactor.\nmod current;\n".into()),
                (
                    BACKEND.into(),
                    "//! Backend.\nenum ReactorBackend { Current }\n".into(),
                ),
                (
                    CONSTRUCTION.into(),
                    "//! Construction.\nfn current() {}\n".into(),
                ),
            ]),
        )
    }

    pub(super) fn at(root: PathBuf, inputs: BTreeMap<String, String>) -> Self {
        assert!(!root.exists(), "never overwrite qualification inputs");
        fs::create_dir_all(&root).expect("create test-owned root before installing cleanup");
        let fixture = Self { root, inputs };
        for (path, text) in &fixture.inputs {
            fixture.write(path, text);
        }
        fixture
    }

    pub(super) fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("test directories");
        fs::write(path, bytes).expect("test source");
    }

    pub(super) fn original_accepts(&self) -> bool {
        let read = |path| fs::read_to_string(self.root.join(path));
        let (Ok(modules), Ok(backend), Ok(construction)) =
            (read(MODULES), read(BACKEND), read(CONSTRUCTION))
        else {
            return false;
        };
        std::panic::catch_unwind(|| {
            super::legacy::check_modules(&super::legacy::modules(&modules));
            super::legacy::check_variant(super::legacy::has_variant(&backend));
            super::physical::check_construction(&construction);
            assert!(
                NAMES
                    .iter()
                    .all(|name| !super::physical::contains_rust_source(
                        &self.root.join("src/reactor").join(name)
                    ))
            );
        })
        .is_ok()
    }

    pub(super) fn hashes(&self) -> BTreeMap<String, String> {
        fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, String>) {
            for entry in fs::read_dir(path).expect("test directory") {
                let entry = entry.expect("test entry");
                if entry.file_type().expect("type").is_dir() {
                    visit(root, &entry.path(), result);
                } else {
                    result.insert(
                        entry
                            .path()
                            .strip_prefix(root)
                            .expect("relative")
                            .to_str()
                            .expect("UTF-8 path")
                            .replace('\\', "/"),
                        sha256_hex(&fs::read(entry.path()).expect("input")),
                    );
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(&self.root, &self.root, &mut result);
        result
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove only test-owned inputs");
    }
}

#[derive(Serialize)]
pub(super) struct Outcome {
    pub(super) case: String,
    pub(super) inputs: BTreeMap<String, String>,
    pub(super) legacy_accepted: bool,
    pub(super) native_accepted: bool,
    pub(super) diagnostics: Vec<(String, String)>,
    pub(super) files: Vec<crate::GovernedRepositoryFile>,
    pub(super) inventories: Vec<crate::GovernedRustInventory>,
}

pub(super) fn observe(fixture: &Fixture, policy: &Fragment, case: &str) -> Outcome {
    let mut row = Outcome {
        case: case.into(),
        inputs: fixture.hashes(),
        legacy_accepted: fixture.original_accepts(),
        native_accepted: true,
        diagnostics: vec![],
        files: vec![],
        inventories: vec![],
    };
    match crate::repository_files::analyze(&fixture.root, &policy.repository.files) {
        Ok(analysis) => {
            row.diagnostics.extend(
                analysis
                    .findings
                    .iter()
                    .map(|finding| (finding.rule.clone(), finding.id.clone())),
            );
            row.files = analysis.policies;
        }
        Err(error) => row.diagnostics.push((
            "repository:files".into(),
            error.split(':').next().expect("diagnostic").into(),
        )),
    }
    let sources = [MODULES, BACKEND]
        .into_iter()
        .filter_map(|path| {
            fs::read_to_string(fixture.root.join(path))
                .ok()
                .map(|source| (path.into(), source))
        })
        .collect();
    for rule in &policy.source.rust.inventories {
        match super::super::native::analyze(&fixture.root, &sources, rule) {
            Ok(observed) => {
                if !observed.satisfied {
                    row.diagnostics
                        .push((observed.policy_id.clone(), "RUST-INVENTORY-001".into()));
                }
                row.inventories.push(observed);
            }
            Err(error) => row.diagnostics.push((
                format!("rust:inventory:{}", rule.name),
                error.split(':').next().expect("diagnostic").into(),
            )),
        }
    }
    row.diagnostics.sort();
    row.native_accepted = row.diagnostics.is_empty();
    assert_eq!(
        row.legacy_accepted, row.native_accepted,
        "{case}: {:?}",
        row.diagnostics
    );
    row
}
