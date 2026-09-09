//! Fixed authored paths discharge a lexical precondition, not arbitrary manifest discovery.

use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use zrail_core::{
    RepositoryEntryMode, RepositoryFilePredicate, RepositoryFileRule, normalize_relative,
    sha256_hex,
};

pub(super) const POLICY: &str = "docs/rc9/policies/kafka-driver.metadata.fragment.toml";
pub(super) const ORIGINS: &str = "crates/zrail-testkit/tests/fixtures/rc9/metadata-origins.json";
pub(super) const HELPER: &str = "crates/zrail-rust/src/repository_files/metadata/legacy.rs";
pub(super) const SOURCE_SHA: &str =
    "336a0780ae3ec984f2a099f695387b0764c8b87303ddb81cdd7322829072576c";
pub(super) const LICENSES: [&str; 3] = [
    "LICENSE",
    "crates/kafka-driver-core/LICENSE",
    "crates/kafka-driver-transport/LICENSE",
];

pub(super) fn project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project")
        .to_owned()
}

pub(super) fn source_binding() {
    let source = fs::read(project().join(HELPER)).expect("original copy");
    let origins: serde_json::Value =
        serde_json::from_slice(&fs::read(project().join(ORIGINS)).expect("origins"))
            .expect("origins JSON");
    for extraction in origins["extractions"]
        .as_array()
        .expect("extractions")
        .iter()
        .take(2)
    {
        let marker = if extraction["start_line"] == 9 {
            "const CANONICAL_REPOSITORY"
        } else {
            "    let root = workspace_root();"
        };
        let text = std::str::from_utf8(&source).expect("UTF-8 original");
        let start = text.find(marker).expect("original start");
        let lines = extraction["end_line"].as_u64().unwrap()
            - extraction["start_line"].as_u64().unwrap()
            + 1;
        let excerpt = text[start..]
            .split_inclusive('\n')
            .take(usize::try_from(lines).expect("bounded excerpt lines"))
            .collect::<String>();
        assert_eq!(
            sha256_hex(excerpt.as_bytes()),
            extraction["extracted_sha256"].as_str().unwrap()
        );
    }
}

pub(super) fn policies() -> Vec<RepositoryFileRule> {
    let document: toml::Value = fs::read_to_string(project().join(POLICY))
        .expect("existing fragment")
        .parse()
        .expect("TOML");
    let policies = document["repository"]["files"]
        .as_array()
        .expect("rules")
        .iter()
        .map(|value| {
            value
                .clone()
                .try_into::<RepositoryFileRule>()
                .expect("typed policy")
        })
        .filter(|rule| rule.name.ends_with("-license-copy"))
        .collect::<Vec<_>>();
    assert!(mapping_valid(&policies), "exact translated license mapping");
    policies
}

pub(super) fn mapping_valid(policies: &[RepositoryFileRule]) -> bool {
    policies.len() == 3
        && policies
            .iter()
            .zip(super::super::PUBLIC_PACKAGES)
            .zip(LICENSES)
            .all(|((rule, (name, _)), license)| {
                rule.name == format!("kd-metadata-{name}-license-copy")
                    && rule.include == [license]
                    && rule.exclude.is_empty()
                    && rule.entry == RepositoryEntryMode::File
                    && rule.predicate
                        == RepositoryFilePredicate::BytesEqual {
                            other: "LICENSE".into(),
                            utf8: true,
                        }
            })
}

#[derive(Serialize)]
pub(super) struct PathProof {
    root_shape: &'static str,
    name: &'static str,
    manifest: &'static str,
    parent: String,
    license: String,
    policy_id: String,
}

pub(super) fn path_proof(root: &Path) -> Vec<PathProof> {
    assert!(root.is_absolute());
    let anchors = [
        (
            "filesystem-root",
            root.ancestors().last().expect("root").to_owned(),
        ),
        ("checkout", root.to_owned()),
        ("nested-unicode", root.join("nested space/λ")),
        ("lexical-only", root.join("uncreated/child")),
    ];
    let mut rows = Vec::new();
    for (root_shape, anchor) in anchors {
        for ((name, manifest), expected) in super::super::PUBLIC_PACKAGES.into_iter().zip(LICENSES)
        {
            assert_eq!(
                normalize_relative(Path::new(manifest)).expect("canonical literal"),
                manifest
            );
            assert_eq!(
                Path::new(manifest).file_name().expect("nonempty filename"),
                "Cargo.toml"
            );
            let joined = anchor.join(manifest);
            let parent = joined
                .parent()
                .expect("fixed joined manifest always has a parent");
            let license = parent.join("LICENSE");
            assert_eq!(license, anchor.join(expected));
            rows.push(PathProof {
                root_shape,
                name,
                manifest,
                parent: normalize_relative(parent.strip_prefix(&anchor).expect("contained parent"))
                    .expect("relative parent"),
                license: normalize_relative(
                    license.strip_prefix(&anchor).expect("contained license"),
                )
                .expect("relative license"),
                policy_id: format!("repository:file:kd-metadata-{name}-license-copy"),
            });
        }
    }
    assert_eq!(rows.len(), 12);
    rows
}

pub(super) fn inputs(root: &Path) -> BTreeMap<String, String> {
    let origins: serde_json::Value =
        serde_json::from_slice(&fs::read(project().join(ORIGINS)).expect("origins")).expect("JSON");
    origins["fixtures"]
        .as_array()
        .expect("fixtures")
        .iter()
        .map(|fixture| {
            let path = fixture["path"].as_str().expect("path");
            let digest = sha256_hex(&fs::read(root.join(path)).expect("complete original inputs"));
            assert_eq!(digest, fixture["sha256"].as_str().expect("frozen digest"));
            (path.to_owned(), digest)
        })
        .collect()
}

pub(super) fn original_and_native(root: &Path) -> Vec<crate::GovernedRepositoryFile> {
    assert_eq!(inputs(root).len(), 11);
    super::super::check(root);
    let rules = policies();
    let result =
        crate::repository_files::analyze(root, &rules).expect("complete native license inputs");
    assert!(result.findings.is_empty(), "{:?}", result.findings);
    assert_eq!(result.policies.len(), 3);
    for (rule, license) in rules.iter().zip(LICENSES) {
        let observed = result
            .policies
            .iter()
            .find(|row| row.policy.name == rule.name)
            .expect("policy identity");
        assert_eq!(observed.policy, *rule);
        assert_eq!(observed.analysis, zrail_core::AnalysisQuality::Exact);
        assert!(observed.satisfied);
        assert_eq!(observed.claim, "utf8-file-bytes");
        assert_eq!(observed.entries.len(), 1);
        assert_eq!(observed.entries[0].path, license);
    }
    result.policies
}
