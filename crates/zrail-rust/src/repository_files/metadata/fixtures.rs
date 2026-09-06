//! Run the complete frozen guard and intended native policy on isolated physical inputs.

use std::{collections::BTreeMap, fs, path::Path};

use zrail_core::{RepositoryFileRule, sha256_hex};

use super::{
    Suite,
    model::{self, FixtureOutcome},
};

pub(super) fn legacy_result(
    root: &Path,
    policy: &RepositoryFileRule,
    suite: &Suite,
) -> Result<(), String> {
    std::panic::catch_unwind(|| (suite.legacy)(root, policy)).map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).into()))
            .expect("original guard's string panic")
    })
}

pub(in super::super) fn qualify(
    root: &Path,
    inputs: &BTreeMap<String, Vec<u8>>,
    policies: &[RepositoryFileRule],
    suite: &Suite,
) -> Vec<FixtureOutcome> {
    assert!(
        !root.exists(),
        "only create a new test-owned fixture directory"
    );
    fs::create_dir(root).expect("own fixture root");
    let _cleanup = Cleanup(root);
    let mut outcomes = Vec::new();
    for policy in policies {
        for case in std::iter::once(None).chain(
            (suite.mutations)(policy, &inputs[&policy.include[0]])
                .into_iter()
                .map(Some),
        ) {
            restore(root, inputs);
            if let Some(case) = &case {
                let path = root.join(&policy.include[0]);
                match &case.bytes {
                    Some(bytes) => fs::write(&path, bytes).expect("mutated input"),
                    None => fs::remove_file(&path).expect("removed input"),
                }
                if case.directory {
                    fs::create_dir(path).expect("wrong entry kind");
                }
            }
            let policy_id = format!("repository:file:{}", policy.name);
            let legacy = legacy_result(root, policy, suite);
            let native = crate::repository_files::analyze(root, std::slice::from_ref(policy));
            let diagnostic = match &native {
                Ok(analysis) => {
                    assert!(
                        analysis
                            .findings
                            .iter()
                            .all(|finding| finding.rule == policy_id)
                    );
                    analysis.findings.first().map(|finding| finding.id.clone())
                }
                Err(error) => {
                    assert!(
                        error.contains("REP-FILE-006") && error.contains(&policy_id),
                        "{error}"
                    );
                    Some("REP-FILE-006".into())
                }
            };
            let expected = case.as_ref().and_then(|case| case.diagnostic);
            assert_eq!(diagnostic.as_deref(), expected, "{policy_id}");
            assert_eq!(
                legacy.is_ok(),
                expected.is_none(),
                "{policy_id}: {legacy:?}"
            );
            let observed: BTreeMap<_, _> = inputs
                .keys()
                .filter_map(|path| {
                    fs::read(root.join(path))
                        .ok()
                        .map(|bytes| (path.clone(), bytes))
                })
                .collect();
            let no_bytes = case.as_ref().is_some_and(|case| case.bytes.is_none());
            assert_eq!(observed.len(), inputs.len() - usize::from(no_bytes));
            assert!(
                observed
                    .iter()
                    .all(|(path, bytes)| *path == policy.include[0] || bytes == &inputs[path]),
                "only the intended input may change"
            );
            outcomes.push(FixtureOutcome {
                policy_id,
                case: case.as_ref().map_or("valid", |case| case.name).into(),
                path: policy.include[0].clone(),
                source_sha256: fs::read(root.join(&policy.include[0]))
                    .ok()
                    .map(|bytes| sha256_hex(&bytes)),
                entry_kind: if case.as_ref().is_some_and(|case| case.directory) {
                    "directory"
                } else if no_bytes {
                    "absent"
                } else {
                    "file"
                }
                .into(),
                inputs: model::hashes(&observed),
                legacy_accepted: legacy.is_ok(),
                legacy_failure: legacy.err(),
                native_accepted: diagnostic.is_none(),
                diagnostic,
            });
        }
    }
    outcomes
}

fn restore(root: &Path, inputs: &BTreeMap<String, Vec<u8>>) {
    for (path, bytes) in inputs {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("contained input parent"))
            .expect("input directories");
        if path.is_dir() {
            fs::remove_dir(&path).expect("remove test-created empty wrong-kind directory");
        }
        fs::write(path, bytes).expect("restore immutable input bytes");
    }
}

struct Cleanup<'a>(&'a Path);

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        fs::remove_dir_all(self.0).expect("remove only the test-owned fixture");
    }
}
