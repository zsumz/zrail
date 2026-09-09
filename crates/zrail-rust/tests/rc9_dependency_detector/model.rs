//! Whole observed name sets and counts distinguish membership from duplicate-line evidence.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use zrail_core::{RepositoryFileRule, sha256_hex};

pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.dependency-detector.fixture.toml";
pub(super) const CASES: &str =
    "crates/zrail-testkit/tests/fixtures/rc9/dependency-detector-cases.json";
pub(super) const ORIGINAL: &str =
    "crates/zrail-rust/tests/rc9_dependency_detector/original_test.rs";
pub(super) const HELPER: &str = "crates/zrail-rust/src/repository_files/raw_dependency/legacy.rs";
pub(super) const SOURCE_SHA: &str =
    "fa3fbc3331aa5fe299f7aa50410093188a6386da4103bb8678d5374a7bfbc53f";
pub(super) const DETECTOR_SHA: &str =
    "1ca7e3fbd5260a785d5d1eb98a4bd0ecdb6b1c0e0b86cd3aa2f5889092df8e66";
pub(super) const HELPER_SHA: &str =
    "023bad9b9e2e98a40249131e5c46e5a436698c210d0e96aa5663b7d089d72d90";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    repository: Rules,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    files: Vec<RepositoryFileRule>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    schema: u32,
    scope: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    source: String,
    names: Vec<String>,
    counts: [usize; 2],
}
#[derive(Serialize)]
pub(super) struct Row {
    case: String,
    source_sha256: String,
    original_names: Vec<String>,
    diagnostics: Vec<(String, String)>,
    observations: Vec<crate::GovernedRepositoryFile>,
}

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("project")
        .to_owned()
}

pub(super) fn source_binding() {
    for (path, marker, expected) in [
        (HELPER, "fn lockfile_packages(", HELPER_SHA),
        (
            ORIGINAL,
            "#[test]\nfn package_extraction_matches_exact_names_only(",
            DETECTOR_SHA,
        ),
    ] {
        let source = fs::read_to_string(project().join(path)).expect("original copy");
        let start = source.find(marker).expect("function");
        let end = start + source[start..].find("\n}\n").expect("function end") + 3;
        assert_eq!(sha256_hex(&source.as_bytes()[start..end]), expected);
    }
}

pub(super) fn qualify() -> Vec<Row> {
    source_binding();
    let fragment: Fragment =
        toml::from_str(&fs::read_to_string(project().join(POLICY)).expect("fixture policy"))
            .expect("strict fixture policy");
    let policies = fragment.repository.files;
    assert_eq!(policies.len(), 2);
    let matrix: Matrix = serde_json::from_slice(&fs::read(project().join(CASES)).expect("cases"))
        .expect("typed cases");
    assert_eq!(matrix.schema, 1);
    assert!(!matrix.scope.is_empty());
    assert_eq!(matrix.cases.len(), 32);
    assert_eq!(matrix.cases[0].name, "original");
    assert_eq!(
        matrix.cases[0].source,
        "[[package]]\nname = \"tokio-util-extra\"\n[[package]]\nname = \"bytes\"\n"
    );
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("external temp")
        .join(format!(
            "zrail-dependency-detector-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
    assert!(!root.exists(), "fresh fixture");
    fs::create_dir(&root).expect("fixture");
    let mut rows = Vec::new();
    for case in matrix.cases {
        let names = super::super::lockfile_packages(&case.source)
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(names, case.names, "{}: complete original names", case.name);
        fs::write(root.join("detector.txt"), &case.source).expect("input");
        let analysis =
            crate::repository_files::analyze(&root, &policies).expect("complete analysis");
        assert_eq!(analysis.policies.len(), 2);
        let expected = [
            !names.iter().any(|name| name == "tokio"),
            names.iter().any(|name| name == "bytes"),
        ];
        let mut diagnostics = Vec::new();
        for (index, observed) in analysis.policies.iter().enumerate() {
            assert_eq!(observed.policy, policies[index]);
            assert_eq!(observed.analysis, zrail_core::AnalysisQuality::Exact);
            assert_eq!(observed.satisfied, expected[index], "{}", case.name);
            assert_eq!(observed.claim, "raw-utf8-lines");
            assert_eq!(observed.entries.len(), 1);
            let entry = &observed.entries[0];
            assert_eq!(entry.literal_count, Some(case.counts[index]));
            assert_eq!(entry.literal_offsets.len(), case.counts[index].min(16));
            assert_eq!(
                entry.omitted_literal_offsets,
                case.counts[index].saturating_sub(16)
            );
            assert_eq!(
                entry.sha256.as_deref(),
                Some(sha256_hex(case.source.as_bytes()).as_str())
            );
            if !expected[index] {
                diagnostics.push((observed.policy_id.clone(), "REP-FILE-004".to_owned()));
            }
        }
        let actual = analysis
            .findings
            .iter()
            .map(|finding| (finding.rule.clone(), finding.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(actual, diagnostics, "{}: intended diagnostic", case.name);
        rows.push(Row {
            case: case.name,
            source_sha256: sha256_hex(case.source.as_bytes()),
            original_names: names,
            diagnostics,
            observations: analysis.policies,
        });
    }
    fs::remove_dir_all(&root).expect("remove test-owned fixture");
    rows
}
