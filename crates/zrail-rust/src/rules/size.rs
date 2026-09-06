//! Source-size design targets, hard ceilings, and tightening ratchets.

use std::collections::{BTreeMap, BTreeSet};

mod ratchet;
mod scoped;
#[cfg(test)]
#[path = "size_test.rs"]
mod size_test;
#[cfg(test)]
pub(crate) use size_test::legacy_driver_paths;

use zrail_core::{Finding, FindingSink, LockedRatchet, Severity, SizeTargetMode};

use crate::source_budget::{self, EffectiveSizeBudget};

use ratchet::check_ratchet;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let ratchets = context
        .contract
        .ratchets
        .iter()
        .filter(|ratchet| ratchet.rule == "rust.file-size")
        .map(|ratchet| (ratchet.target.as_str(), ratchet))
        .collect::<BTreeMap<_, _>>();
    let locked = context.lock.map_or_else(BTreeMap::new, |lock| {
        lock.ratchets
            .iter()
            .filter(|ratchet| ratchet.rule == "rust.file-size")
            .map(|ratchet| (ratchet.target.as_str(), ratchet))
            .collect()
    });
    let mut seen = BTreeSet::new();
    for file in &context.source.files {
        if !seen.insert(file.relative.as_str()) {
            continue;
        }
        let budget = match source_budget::for_file(file, &context.contract.source.rust) {
            Ok(budget) => budget,
            Err(message) => {
                findings.push(
                    Finding::error("RUST-SIZE-006", "rust.file-size", "source-size", message)
                        .at(&file.relative, None),
                );
                continue;
            }
        };
        check_file(
            file,
            budget.as_ref(),
            ratchets.get(file.relative.as_str()).copied(),
            locked.get(file.relative.as_str()).copied(),
            findings,
        );
    }
    scoped::missing_exceptions(context, &seen, findings);
    for target in ratchets.keys() {
        if !seen.contains(target) {
            findings.push(
                Finding::error(
                    "RUST-SIZE-005",
                    "rust.file-size",
                    "source-size",
                    format!("file-size ratchet names missing source {target:?}"),
                )
                .at(*target, None),
            );
        }
    }
}

fn check_file(
    file: &crate::source::RustFileFacts,
    budget: Option<&EffectiveSizeBudget>,
    contract_ratchet: Option<&zrail_core::RatchetContract>,
    locked_ratchet: Option<&LockedRatchet>,
    findings: &mut FindingSink,
) {
    let Some(budget) = budget else {
        if contract_ratchet.is_some() || locked_ratchet.is_some() {
            findings.push(
                Finding::error(
                    "RUST-SIZE-005",
                    "rust.file-size",
                    "source-size",
                    "file-size ratchet has no active size policy",
                )
                .at(&file.relative, None)
                .with_help("remove the stale ratchet or restore a reviewed size policy"),
            );
        }
        return;
    };
    scoped::check(file, budget, findings);
    if file.lines <= budget.thresholds.target {
        if contract_ratchet.is_some() || locked_ratchet.is_some() {
            findings.push(
                Finding::error(
                    "RUST-SIZE-004",
                    "rust.file-size.ratchet",
                    "source-size",
                    "file returned below its design target but retains a stale ratchet",
                )
                .at(&file.relative, None)
                .with_help("remove the ratchet from zrail.toml and run `zrail update`"),
            );
        }
        return;
    }
    if let Some(contract_ratchet) = contract_ratchet {
        check_ratchet(file, contract_ratchet, locked_ratchet, findings);
        return;
    }
    if !budget.independent_hard && file.lines > budget.thresholds.hard {
        findings.push(
            Finding::error(
                "RUST-SIZE-001",
                "rust.file-size.hard",
                "source-size",
                format!(
                    "source is {} lines, above its absolute {}-line ceiling",
                    file.lines, budget.thresholds.hard
                ),
            )
            .at(&file.relative, None)
            .with_help("split the responsibility at a semantic module boundary"),
        );
    }
    findings.push(
        Finding::error(
            "RUST-SIZE-002",
            "rust.file-size.target",
            "source-size",
            format!(
                "source is {} lines, above its {}-line design target",
                file.lines, budget.thresholds.target
            ),
        )
        .at(&file.relative, None)
        .with_severity(match budget.thresholds.target_mode {
            SizeTargetMode::Error => Severity::Error,
            SizeTargetMode::Warn => Severity::Warning,
        })
        .with_help(format!(
            "{}: split the file or add a reviewed ratchet with a concrete reason",
            budget.policy_id
        )),
    );
}
