//! Reusable exact per-file count debt that can only tighten.

mod value;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    FindingSink, LintSuppressionMode, ModuleDocsMode, PolicyMode, RustSourceContract,
};

use crate::{
    inventory::FileClass,
    source::{RustFileFacts, SourceSyntax},
};

use super::RuleContext;

#[derive(Clone, Copy)]
pub(super) struct CountRatchetSpec<'a> {
    pub(super) rule: &'static str,
    pub(super) finding_id: &'static str,
    pub(super) finding_rule: &'static str,
    pub(super) category: &'static str,
    pub(super) debt: &'a str,
    pub(super) report_source_lock_drift: bool,
}

pub(super) fn evaluate(
    context: &RuleContext<'_>,
    spec: CountRatchetSpec<'_>,
    selector: Option<&str>,
    findings: &mut FindingSink,
    report_unratcheted: impl Fn(&RustFileFacts, &mut FindingSink),
) {
    let ratchets = context
        .contract
        .ratchets
        .iter()
        .filter(|ratchet| {
            ratchet.rule == spec.rule && selector_matches(ratchet.selector.as_deref(), selector)
        })
        .map(|ratchet| (ratchet.target.as_str(), ratchet))
        .collect::<BTreeMap<_, _>>();
    let locked = context.lock.map_or_else(BTreeMap::new, |lock| {
        lock.ratchets
            .iter()
            .filter(|ratchet| {
                ratchet.rule == spec.rule && selector_matches(ratchet.selector.as_deref(), selector)
            })
            .map(|ratchet| (ratchet.target.as_str(), ratchet))
            .collect()
    });
    let mut seen = BTreeSet::new();
    for file in &context.source.files {
        let Some(value) = measurement(spec.rule, selector, file, &context.contract.source.rust)
        else {
            continue;
        };
        seen.insert(file.relative.as_str());
        value::check_value(
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
        if spec.report_source_lock_drift {
            findings.push(value::ratchet_finding(
                spec,
                target,
                format!(
                    "{} ratchet names missing governed source {target:?}",
                    spec.debt
                ),
            ));
        }
    }
}

pub(crate) fn measurement(
    rule: &str,
    selector: Option<&str>,
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
        "rust.hygiene.denied-method" => file
            .reachability
            .is_non_test_target()
            .then(|| {
                let selector = selector?;
                Some(
                    file.methods
                        .iter()
                        .filter(|method| method_matches(selector, method))
                        .count(),
                )
            })
            .flatten(),
        "rust.hygiene.denied-macro" => file
            .reachability
            .is_non_test_target()
            .then(|| {
                let selector = selector?;
                Some(
                    file.macros
                        .iter()
                        .filter(|invocation| macro_matches(selector, invocation))
                        .count(),
                )
            })
            .flatten(),
        _ => None,
    }
}

fn selector_matches(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            zrail_core::normalize_ratchet_selector(left)
                == zrail_core::normalize_ratchet_selector(right)
        }
        _ => false,
    }
}

pub(super) fn method_matches(selector: &str, method: &crate::source::ObservedFact) -> bool {
    zrail_core::normalize_ratchet_selector(selector)
        == zrail_core::normalize_ratchet_selector(&method.name)
}

pub(super) fn macro_matches(selector: &str, invocation: &crate::source::ObservedFact) -> bool {
    let selector = zrail_core::normalize_ratchet_selector(selector);
    invocation.policy_names().any(|name| {
        let name = zrail_core::normalize_ratchet_selector(name);
        name == selector
            || name
                .rsplit("::")
                .next()
                .is_some_and(|leaf| leaf == selector)
    })
}

pub(super) fn lint_suppression_violates(
    mode: LintSuppressionMode,
    suppression: &crate::source::ObservedFact,
) -> bool {
    mode == LintSuppressionMode::Deny || suppression.name == "unreasoned lint suppression"
}
