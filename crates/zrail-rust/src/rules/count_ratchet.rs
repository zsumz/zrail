//! Reusable exact per-file count debt that can only tighten.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    Finding, FindingSink, LintSuppressionMode, LockedRatchet, ModuleDocsMode, PolicyMode,
    RatchetContract, RustSourceContract,
};

use crate::{
    inventory::FileClass,
    source::{RustFileFacts, SourceSyntax},
};

use super::RuleContext;

#[derive(Clone, Copy)]
pub(super) struct CountRatchetSpec {
    pub(super) rule: &'static str,
    pub(super) finding_id: &'static str,
    pub(super) finding_rule: &'static str,
    pub(super) category: &'static str,
    pub(super) debt: &'static str,
}

pub(super) fn evaluate(
    context: &RuleContext<'_>,
    spec: CountRatchetSpec,
    findings: &mut FindingSink,
    report_unratcheted: impl Fn(&RustFileFacts, &mut FindingSink),
) {
    let ratchets = context
        .contract
        .ratchets
        .iter()
        .filter(|ratchet| ratchet.rule == spec.rule)
        .map(|ratchet| (ratchet.target.as_str(), ratchet))
        .collect::<BTreeMap<_, _>>();
    let locked = context.lock.map_or_else(BTreeMap::new, |lock| {
        lock.ratchets
            .iter()
            .filter(|ratchet| ratchet.rule == spec.rule)
            .map(|ratchet| (ratchet.target.as_str(), ratchet))
            .collect()
    });
    let mut seen = BTreeSet::new();
    for file in &context.source.files {
        let Some(value) = measurement(spec.rule, file, &context.contract.source.rust) else {
            continue;
        };
        seen.insert(file.relative.as_str());
        check_value(
            file,
            value,
            ratchets.get(file.relative.as_str()).copied(),
            locked.get(file.relative.as_str()).copied(),
            spec,
            findings,
            &report_unratcheted,
        );
    }
    for target in ratchets.keys().filter(|target| !seen.contains(*target)) {
        findings.push(ratchet_finding(
            spec,
            target,
            format!(
                "{} ratchet names missing governed source {target:?}",
                spec.debt
            ),
        ));
    }
}

pub(crate) fn measurement(
    rule: &str,
    file: &RustFileFacts,
    rust: &RustSourceContract,
) -> Option<usize> {
    match rule {
        "rust.file-size" => Some(file.lines),
        "rust.inline-tests" => file
            .reachability
            .is_non_test_target()
            .then_some(file.tests.len()),
        "rust.module-docs" => (rust.module_docs == ModuleDocsMode::Required
            && file.syntax == SourceSyntax::Items
            && file.class != FileClass::Generated)
            .then_some(usize::from(!file.module_docs)),
        "rust.hygiene.unsafe" => (rust.hygiene.unsafe_code == PolicyMode::Deny
            && file.reachability.is_non_test_target())
        .then_some(file.unsafe_constructs.len()),
        "rust.hygiene.lint-suppressions" => (rust.hygiene.lint_suppressions
            != LintSuppressionMode::Allow
            && file.reachability.is_non_test_target()
            && file.class != FileClass::Generated)
            .then(|| {
                file.lint_suppressions
                    .iter()
                    .filter(|suppression| {
                        lint_suppression_violates(rust.hygiene.lint_suppressions, suppression)
                    })
                    .count()
            }),
        _ => None,
    }
}

pub(super) fn lint_suppression_violates(
    mode: LintSuppressionMode,
    suppression: &crate::source::ObservedFact,
) -> bool {
    mode == LintSuppressionMode::Deny || suppression.name == "unreasoned lint suppression"
}

fn check_value(
    file: &RustFileFacts,
    value: usize,
    ratchet: Option<&RatchetContract>,
    locked: Option<&LockedRatchet>,
    spec: CountRatchetSpec,
    findings: &mut FindingSink,
    report_unratcheted: &impl Fn(&RustFileFacts, &mut FindingSink),
) {
    if value == 0 {
        if ratchet.is_some() {
            findings.push(
                ratchet_finding(
                    spec,
                    &file.relative,
                    format!(
                        "{} was removed but the source retains a stale ratchet",
                        spec.debt
                    ),
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
                spec,
                &file.relative,
                format!("reviewed {} ratchet is absent from zrail.lock", spec.debt),
            )
            .because(&ratchet.reason)
            .with_help("run `zrail update` and review the generated debt"),
        );
        return;
    };
    if value > locked.value {
        findings.push(
            ratchet_finding(
                spec,
                &file.relative,
                format!(
                    "{} grew from the {}-construct ratchet to {value}",
                    spec.debt, locked.value
                ),
            )
            .because(&ratchet.reason),
        );
    } else if value < locked.value {
        findings.push(
            ratchet_finding(
                spec,
                &file.relative,
                format!(
                    "{} shrank to {value} constructs but the lock still permits {}",
                    spec.debt, locked.value
                ),
            )
            .with_help("run `zrail update` to tighten the recorded debt"),
        );
    }
}

fn ratchet_finding(spec: CountRatchetSpec, path: &str, message: String) -> Finding {
    Finding::error(spec.finding_id, spec.finding_rule, spec.category, message).at(path, None)
}
