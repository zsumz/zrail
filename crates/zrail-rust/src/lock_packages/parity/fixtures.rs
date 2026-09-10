//! Every mutated lock remains a complete native Cargo graph and reaches its intended inventory rail.

use std::{collections::BTreeMap, fs, path::Path};

use zrail_core::{FindingSink, LockPackageRule, sha256_hex};

use super::{
    legacy,
    model::{self, FixtureOutcome},
    mutations,
};

pub(super) fn qualify(
    root: &Path,
    inputs: &BTreeMap<String, Vec<u8>>,
    rules: &[LockPackageRule],
) -> Vec<FixtureOutcome> {
    assert!(!root.exists(), "only create a new test-owned fixture root");
    fs::create_dir(root).expect("own fixture root");
    let _cleanup = Cleanup(root);
    for (path, bytes) in inputs {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("input parent")).expect("input directories");
        fs::write(path, bytes).expect("frozen input copy");
    }
    let workspace = model::workspace(root);
    let mut outcomes = Vec::new();
    for rule in rules {
        let cases = std::iter::once(("valid".into(), inputs["Cargo.lock"].clone(), true))
            .chain(mutations::cases(&inputs["Cargo.lock"], &rule.package));
        for (case, bytes, accepted) in cases {
            fs::write(root.join("Cargo.lock"), &bytes).expect("mutated lock");
            let legacy = legacy_result(root);
            let observed = model::native(root, &workspace, std::slice::from_ref(rule))
                .expect("mutation preserves complete Cargo resolution")
                .remove(0);
            let mut findings = FindingSink::default();
            crate::lock_packages::evaluate(std::slice::from_ref(&observed), &mut findings);
            let findings = findings.into_findings();
            let diagnostic = findings.first().map(|finding| finding.id.clone());
            assert_eq!(
                diagnostic.as_deref(),
                if accepted { None } else { Some("DEP-LOCK-001") },
                "{}:{case}",
                rule.name
            );
            assert!(
                findings
                    .iter()
                    .all(|finding| finding.rule == observed.policy_id)
            );
            assert_eq!(observed.satisfied, accepted, "{}:{case}", rule.name);
            assert_eq!(legacy.is_ok(), accepted, "{}:{case}: {legacy:?}", rule.name);
            let actual = model::inputs(root);
            assert!(
                actual
                    .iter()
                    .all(|(path, value)| path == "Cargo.lock" || value == &inputs[path])
            );
            outcomes.push(FixtureOutcome {
                policy_id: observed.policy_id.clone(),
                case,
                path: "Cargo.lock".into(),
                source_sha256: sha256_hex(&bytes),
                inputs: model::hashes(&actual),
                legacy_accepted: legacy.is_ok(),
                legacy_failure: legacy.err(),
                native_accepted: observed.satisfied,
                diagnostic,
                observed,
            });
        }
    }
    outcomes
}

pub(super) fn legacy_result(root: &Path) -> Result<(), String> {
    std::panic::catch_unwind(|| legacy::check(root)).map_err(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).into()))
            .expect("frozen panic message")
    })
}

struct Cleanup<'a>(&'a Path);

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        fs::remove_dir_all(self.0).expect("remove test-owned fixture");
    }
}
