//! Trusted comparisons execute the original assertion and the intended native diagnostic.

use std::{collections::BTreeMap, fs, path::Path};

use zrail_core::{FindingSink, RustInventoryRule};

use super::{
    family::Family,
    model::{self, Fixture},
    native,
};

pub(super) fn qualify(
    root: &Path,
    roots: &[String],
    sources: &BTreeMap<String, String>,
    policy: &RustInventoryRule,
) -> Vec<Fixture> {
    qualify_family(root, roots, sources, policy, Family::Methods)
}

pub(super) fn qualify_family(
    root: &Path,
    roots: &[String],
    sources: &BTreeMap<String, String>,
    policy: &RustInventoryRule,
    family: Family,
) -> Vec<Fixture> {
    let policy_id = format!("rust:inventory:kd-transport-{}", family.name());
    assert!(
        !root.exists(),
        "never overwrite an existing qualification directory"
    );
    for directory in roots {
        fs::create_dir_all(root.join(directory)).expect("fixture roots");
    }
    for (path, source) in sources {
        write(root, path, source);
    }
    let mut results = Vec::new();
    for mutation in family.cases(sources, roots) {
        let mut inputs = sources.clone();
        for (path, source) in &mutation.changes {
            if let Some(source) = source {
                inputs.insert(path.clone(), source.clone());
                write(root, path, source);
            } else {
                inputs.remove(path);
                fs::remove_file(root.join(path)).expect("remove fixture input");
            }
        }
        let legacy_fixture_parses = mutation
            .changes
            .iter()
            .filter_map(|(path, source)| {
                source.as_ref().map(|source| {
                    (
                        path.clone(),
                        std::panic::catch_unwind(|| family.observed(path, source)).is_ok(),
                    )
                })
            })
            .collect();
        let legacy_counts = std::panic::catch_unwind(|| family.repository(root, roots)).ok();
        let legacy_accepted = legacy_counts.as_ref().is_some_and(|counts| {
            std::panic::catch_unwind(|| family.check(counts.clone())).is_ok()
        });
        let observed = native::analyze(root, &inputs, policy);
        let native_accepted = observed.as_ref().is_ok_and(|report| report.satisfied);
        assert_eq!(
            legacy_accepted, native_accepted,
            "unaccounted parity difference: {}",
            mutation.name
        );
        let diagnostic = if let Ok(report) = &observed {
            if let Some(counts) = &legacy_counts {
                let native_counts = report
                    .counts
                    .iter()
                    .map(|count| (format!("{}:{}", count.path, count.name), count.count))
                    .collect::<BTreeMap<_, _>>();
                assert_eq!(
                    &native_counts, counts,
                    "complete identity/quantity map: {}",
                    mutation.name
                );
            }
            let mut findings = FindingSink::default();
            crate::rust_inventories::evaluate(std::slice::from_ref(report), &mut findings);
            let findings = findings.into_findings();
            if native_accepted {
                assert!(findings.is_empty());
                None
            } else {
                assert_eq!(findings.len(), 1);
                assert_eq!(findings[0].id, "RUST-INVENTORY-001");
                assert_eq!(findings[0].rule, policy_id);
                Some(findings[0].id.clone())
            }
        } else {
            assert!(
                legacy_counts.is_none(),
                "native-only incomplete analysis: {:?}",
                observed.as_ref().err()
            );
            let error = observed.as_ref().expect_err("incomplete");
            assert!(error.contains("RUST-INVENTORY-002"), "{error}");
            assert!(error.contains(&policy_id), "{error}");
            Some("RUST-INVENTORY-002".into())
        };
        let changed = mutation
            .changes
            .iter()
            .filter_map(|(path, source)| {
                source.as_ref().map(|source| (path.clone(), source.clone()))
            })
            .collect();
        results.push(Fixture {
            case: mutation.name,
            policy_id: policy_id.clone(),
            changed_inputs: model::hashes(&changed),
            removed_inputs: mutation
                .changes
                .iter()
                .filter(|(_, source)| source.is_none())
                .map(|(path, _)| path.clone())
                .collect(),
            legacy_fixture_parses,
            legacy_counts,
            native: observed.ok(),
            legacy_accepted,
            native_accepted,
            diagnostic,
        });
        for path in mutation.changes.keys() {
            if let Some(source) = sources.get(path) {
                write(root, path, source);
            } else {
                fs::remove_file(root.join(path)).expect("remove introduced fixture");
            }
        }
    }
    fs::remove_dir_all(root).expect("remove test-owned fixture directory");
    results
}

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("source parent")).expect("fixture source directory");
    fs::write(path, source).expect("fixture source bytes");
}
