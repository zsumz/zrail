//! Physical helper qualification never substitutes for full Rust or downstream policy analysis.

use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    fs,
    path::{Path, PathBuf},
};
use zrail_core::{RepositoryFileRule, sha256_hex};

pub(super) const ROOTS: [&str; 6] = [
    "src",
    "tests",
    "crates/kafka-driver-core/src",
    "crates/kafka-driver-probe/src",
    "crates/kafka-driver-sim/src",
    "crates/kafka-driver-transport/src",
];
pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.traversal.fragment.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: Repository,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Repository {
    files: Vec<RepositoryFileRule>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project")
        .to_owned()
}

pub(super) fn policies() -> Vec<RepositoryFileRule> {
    let value: Fragment =
        toml::from_str(&fs::read_to_string(project().join(POLICY)).expect("proposal"))
            .expect("strict proposal");
    assert_eq!(value.repository.files.len(), 2);
    value.repository.files
}

pub(super) fn original(root: &Path) -> Vec<String> {
    crate::rules::legacy_driver_paths(root, &ROOTS.map(String::from))
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .expect("relative path")
                .to_str()
                .expect("UTF-8 fixture")
                .replace('\\', "/")
        })
        .collect()
}

pub(super) fn panic_text(error: &(dyn Any + Send)) -> String {
    error
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| error.downcast_ref::<&str>().map(|text| (*text).into()))
        .expect("string panic")
}

pub(super) struct Fixture {
    pub(super) root: PathBuf,
}
impl Fixture {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir()
            .canonicalize()
            .expect("external temp")
            .join(format!(
                "zrail-traversal-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
        assert!(!root.exists(), "fresh fixture");
        fs::create_dir(&root).expect("fixture");
        let value = Self { root };
        for path in ROOTS {
            fs::create_dir_all(value.root.join(path)).expect("selected root");
        }
        value
    }
    pub(super) fn write(&self, relative: &str, bytes: impl AsRef<[u8]>) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("parents");
        fs::write(path, bytes).expect("fixture file");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove test-owned fixture");
    }
}

#[derive(Clone, Serialize)]
pub(super) struct Row {
    pub(super) case: String,
    pub(super) legacy_paths: Vec<String>,
    pub(super) legacy_error: Option<String>,
    pub(super) native_paths: Vec<String>,
    pub(super) native_error: Option<String>,
    pub(super) diagnostics: Vec<(String, String)>,
    pub(super) observations: Vec<crate::GovernedRepositoryFile>,
}

pub(super) fn observe(
    fixture: &Fixture,
    case: &str,
    legacy_error: Option<&str>,
    diagnostic: &str,
) -> Row {
    observe_with(fixture, case, legacy_error, diagnostic, &policies())
}

pub(super) type Observer = fn(&Fixture, &str, Option<&str>, &str) -> Row;

pub(super) fn observe_with(
    fixture: &Fixture,
    case: &str,
    legacy_error: Option<&str>,
    diagnostic: &str,
    policies: &[RepositoryFileRule],
) -> Row {
    let root = &fixture.root;
    let original = std::panic::catch_unwind(|| original(root));
    let (legacy_paths, actual_error) = match original {
        Ok(paths) => (paths, None),
        Err(error) => (
            vec![],
            Some(panic_text(error.as_ref()).replace(root.to_str().expect("root"), "$ROOT")),
        ),
    };
    assert_eq!(actual_error.is_some(), legacy_error.is_some(), "{case}");
    if let Some(prefix) = legacy_error {
        assert!(
            actual_error.as_ref().expect("panic").starts_with(prefix),
            "{case}"
        );
        assert!(
            error_names_root(actual_error.as_ref().expect("panic"), case),
            "intended original root"
        );
    }
    let mut row = Row {
        case: case.into(),
        legacy_paths,
        legacy_error: actual_error,
        native_paths: vec![],
        native_error: None,
        diagnostics: vec![],
        observations: vec![],
    };
    match crate::repository_files::analyze(root, policies) {
        Ok(analysis) => {
            row.diagnostics = analysis
                .findings
                .iter()
                .map(|finding| (finding.rule.clone(), finding.id.clone()))
                .collect();
            row.native_paths = analysis
                .policies
                .iter()
                .filter(|rule| rule.policy_id.ends_with("kd-source-traversal"))
                .flat_map(|rule| &rule.entries)
                .filter(|entry| {
                    entry.kind != "directory"
                        && Path::new(&entry.path)
                            .extension()
                            .is_some_and(|ext| ext == "rs")
                })
                .map(|entry| entry.path.clone())
                .collect();
            row.observations = analysis.policies;
            let expected = if diagnostic.is_empty() {
                vec![]
            } else {
                vec![("repository:file:kd-source-roots".into(), diagnostic.into())]
            };
            assert_eq!(row.diagnostics, expected, "{case}: intended diagnostic");
            if diagnostic.is_empty() {
                compare_paths(&row.legacy_paths, &row.native_paths);
            }
        }
        Err(error) => {
            assert_eq!(diagnostic, "REP-FILE-006", "{case}: {error}");
            assert!(error.starts_with("REP-FILE-006:"));
            assert!(
                error_names_root(&error, case),
                "intended unread boundary: {error}"
            );
            row.native_error = Some(error.replace(root.to_str().expect("root"), "$ROOT"));
        }
    }
    assert_eq!(
        row.native_error.is_some(),
        diagnostic == "REP-FILE-006",
        "{case}"
    );
    row
}

pub(super) fn source_binding() {
    let source = fs::read_to_string(
        project().join("crates/zrail-rust/src/rules/size/snapshot/legacy_driver.rs"),
    )
    .expect("existing oracle");
    let start = source
        .find("fn collect_rust_files(")
        .expect("original function");
    let length = source[start..].find("\n}\n").expect("function end") + 3;
    assert_eq!(
        sha256_hex(&source.as_bytes()[start..start + length]),
        "b2cd5030320fbd1bb6e958d71ae82fa2e0a505b9d8842144f8528450fc25388a"
    );
}

pub(super) fn error_names_root(error: &str, case: &str) -> bool {
    // Compare platform renderings without rewriting the recorded OS error.
    error
        .replace('\\', "/")
        .contains(case.split(':').next().expect("case root"))
}

pub(super) fn compare_paths(original: &[String], native: &[String]) {
    let mut original = original.iter().map(Path::new).collect::<Vec<_>>();
    let mut native = native.iter().map(Path::new).collect::<Vec<_>>();
    original.sort();
    native.sort();
    assert_eq!(
        original, native,
        "complete multiset; no path deduplication or ordering equivalence claim"
    );
}
