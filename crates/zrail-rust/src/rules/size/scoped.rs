//! Independent hard enforcement, soft warnings, and active/stale exact exceptions.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, Severity};

use crate::{source::RustFileFacts, source_budget::EffectiveSizeBudget};

use super::super::RuleContext;

pub(super) fn check(
    file: &RustFileFacts,
    budget: &EffectiveSizeBudget,
    findings: &mut FindingSink,
) {
    if let Some(soft) = budget.thresholds.soft.filter(|soft| file.lines > *soft) {
        findings.push(
            finding(
                file,
                budget,
                "RUST-SIZE-007",
                "rust.file-size.soft",
                &format!(
                    "source is {} lines, above its {soft}-line soft threshold",
                    file.lines
                ),
            )
            .with_severity(Severity::Warning),
        );
    }
    if !budget.independent_hard {
        return;
    }
    if let Some(exception) = &budget.exception {
        if exception.hard <= budget.thresholds.hard || file.lines <= budget.thresholds.hard {
            findings.push(finding(file, budget, "RUST-SIZE-008", "rust.file-size.exception",
                &format!("hard exception is stale: source is {} lines; policy hard {}, exception hard {}",
                    file.lines, budget.thresholds.hard, exception.hard))
                .because(&exception.reason)
                .with_help("remove the exact exception when debt returns below the policy hard ceiling"));
        } else {
            findings.push(finding(file, budget, "RUST-SIZE-010", "rust.file-size.exception",
                &format!("source carries {} lines of above-hard debt, bounded at {}; owner {}; issue {}; tracking {}",
                    file.lines - budget.thresholds.hard, exception.hard,
                    exception.owner.as_deref().unwrap_or("<none>"),
                    exception.issue.as_deref().unwrap_or("<none>"),
                    exception.tracking.as_deref().unwrap_or("<none>")))
                .because(&exception.reason).with_severity(Severity::Warning));
        }
    }
    if file.lines > budget.hard_ceiling() {
        findings.push(finding(file, budget, "RUST-SIZE-001", "rust.file-size.hard",
            &format!("source is {} lines, above its absolute {}-line ceiling", file.lines, budget.hard_ceiling()))
            .with_help("split the source; a measured ratchet or ordinary lock update cannot grant above-hard authority"));
    }
}

pub(super) fn missing_exceptions(
    context: &RuleContext<'_>,
    seen: &BTreeSet<&str>,
    findings: &mut FindingSink,
) {
    for exception in context
        .contract
        .source
        .rust
        .budgets
        .iter()
        .flat_map(|policy| &policy.exceptions)
    {
        if !seen.contains(exception.path.as_str()) {
            findings.push(
                Finding::error(
                    "RUST-SIZE-008",
                    "rust.file-size.exception",
                    "source-size",
                    format!(
                        "rust:size:exception:{} names missing or inactive source",
                        exception.path
                    ),
                )
                .at(&exception.path, None)
                .because(&exception.reason)
                .with_help("remove the stale exact-file exception"),
            );
        }
    }
}

fn finding(
    file: &RustFileFacts,
    budget: &EffectiveSizeBudget,
    id: &str,
    rule: &str,
    message: &str,
) -> Finding {
    let finding = Finding::error(
        id,
        rule,
        "source-size",
        format!("{}: {message}", budget.policy_id),
    )
    .at(&file.relative, None);
    if let Some(scope) = &budget.scoped {
        finding.because(&scope.reason)
    } else {
        finding
    }
}
