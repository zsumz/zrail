//! Source-size design targets, hard ceilings, and tightening ratchets.

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{Budget, Finding, FindingSink, LockedRatchet};

use crate::source_policy;

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
        seen.insert(file.relative.as_str());
        let budget = source_policy::budget_for(
            &file.relative,
            file.class,
            file.reachability,
            &context.contract.source.rust,
        );
        check_file(
            file,
            budget,
            ratchets.get(file.relative.as_str()).copied(),
            locked.get(file.relative.as_str()).copied(),
            findings,
        );
    }
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
    budget: Option<Budget>,
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
    if file.lines > budget.hard {
        findings.push(
            Finding::error(
                "RUST-SIZE-001",
                "rust.file-size.hard",
                "source-size",
                format!(
                    "source is {} lines, above its absolute {}-line ceiling",
                    file.lines, budget.hard
                ),
            )
            .at(&file.relative, None)
            .with_help("split the responsibility at a semantic module boundary"),
        );
    }
    if file.lines <= budget.target {
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
    let Some(contract_ratchet) = contract_ratchet else {
        findings.push(
            Finding::error(
                "RUST-SIZE-002",
                "rust.file-size.target",
                "source-size",
                format!(
                    "source is {} lines, above its {}-line design target",
                    file.lines, budget.target
                ),
            )
            .at(&file.relative, None)
            .with_help("split the file or add a reviewed ratchet with a concrete reason"),
        );
        return;
    };
    let Some(locked_ratchet) = locked_ratchet else {
        findings.push(
            Finding::error(
                "RUST-SIZE-003",
                "rust.file-size.ratchet",
                "source-size",
                "reviewed ratchet is absent from zrail.lock",
            )
            .at(&file.relative, None)
            .because(&contract_ratchet.reason)
            .with_help("run `zrail update` and review the generated debt"),
        );
        return;
    };
    if file.lines > locked_ratchet.value {
        findings.push(
            Finding::error(
                "RUST-SIZE-003",
                "rust.file-size.ratchet",
                "source-size",
                format!(
                    "source grew from its {}-line ratchet to {} lines",
                    locked_ratchet.value, file.lines
                ),
            )
            .at(&file.relative, None)
            .because(&contract_ratchet.reason),
        );
    } else if file.lines < locked_ratchet.value {
        findings.push(
            Finding::error(
                "RUST-SIZE-004",
                "rust.file-size.ratchet",
                "source-size",
                format!(
                    "source shrank to {} lines but lock still permits {}",
                    file.lines, locked_ratchet.value
                ),
            )
            .at(&file.relative, None)
            .with_help("run `zrail update` to tighten the recorded ceiling"),
        );
    }
}
