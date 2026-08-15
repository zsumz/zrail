//! Inline-test debt is exact per production file and can only tighten.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Finding, FindingSink, LockedRatchet, RatchetContract};

use crate::source::RustFileFacts;

use super::super::RuleContext;

pub(super) fn check(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let ratchets = context
        .contract
        .ratchets
        .iter()
        .filter(|ratchet| ratchet.rule == "rust.inline-tests")
        .map(|ratchet| (ratchet.target.as_str(), ratchet))
        .collect::<BTreeMap<_, _>>();
    let locked = context.lock.map_or_else(BTreeMap::new, |lock| {
        lock.ratchets
            .iter()
            .filter(|ratchet| ratchet.rule == "rust.inline-tests")
            .map(|ratchet| (ratchet.target.as_str(), ratchet))
            .collect()
    });
    let mut seen = BTreeSet::new();
    for file in context
        .source
        .files
        .iter()
        .filter(|file| file.reachability.is_production())
    {
        seen.insert(file.relative.as_str());
        check_file(
            file,
            ratchets.get(file.relative.as_str()).copied(),
            locked.get(file.relative.as_str()).copied(),
            findings,
        );
    }
    for target in ratchets.keys() {
        if !seen.contains(target) {
            findings.push(ratchet_finding(
                target,
                &format!("inline-test ratchet names missing production source {target:?}"),
            ));
        }
    }
}

fn check_file(
    file: &RustFileFacts,
    ratchet: Option<&RatchetContract>,
    locked: Option<&LockedRatchet>,
    findings: &mut FindingSink,
) {
    if file.tests.is_empty() {
        if ratchet.is_some() || locked.is_some() {
            findings.push(
                ratchet_finding(
                    &file.relative,
                    "inline tests were removed but the source retains a stale ratchet",
                )
                .with_help("remove the ratchet from zrail.toml and run `zrail update`"),
            );
        }
        return;
    }
    let Some(ratchet) = ratchet else {
        report_unratcheted(file, findings);
        return;
    };
    let Some(locked) = locked else {
        findings.push(
            ratchet_finding(
                &file.relative,
                "reviewed inline-test ratchet is absent from zrail.lock",
            )
            .because(&ratchet.reason),
        );
        return;
    };
    if file.tests.len() > locked.value {
        findings.push(
            ratchet_finding(
                &file.relative,
                &format!(
                    "inline tests grew from the {}-construct ratchet to {}",
                    locked.value,
                    file.tests.len()
                ),
            )
            .because(&ratchet.reason),
        );
    } else if file.tests.len() < locked.value {
        findings.push(
            ratchet_finding(
                &file.relative,
                &format!(
                    "inline tests shrank to {} constructs but the lock still permits {}",
                    file.tests.len(),
                    locked.value
                ),
            )
            .with_help("run `zrail update` to tighten the recorded debt"),
        );
    }
}

fn report_unratcheted(file: &RustFileFacts, findings: &mut FindingSink) {
    for test in &file.tests {
        findings.push(
            Finding::error(
                "RUST-TEST-001",
                "rust.tests.sibling",
                "test-placement",
                format!("production source contains test construct {}", test.name),
            )
            .at(&file.relative, test.span)
            .with_help("move the proof into a reachable sibling `_test.rs` module"),
        );
    }
}

fn ratchet_finding(path: &str, message: &str) -> Finding {
    Finding::error(
        "RUST-TEST-005",
        "rust.tests.inline.ratchet",
        "test-placement",
        message,
    )
    .at(path, None)
}
