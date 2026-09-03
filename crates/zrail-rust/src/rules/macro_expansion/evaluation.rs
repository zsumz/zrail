//! Macro allowance enforcement and stale-authority detection.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, MacroExpansionAllow, MacroExpansionMode, MacroInputMode};

use super::super::RuleContext;
use super::{
    allowances::AllowanceIndex,
    diagnostics::{unbound, unreviewed},
    policy::directly_inspected,
    review::{MacroBindingResult, review},
};

pub(crate) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let enforcing = context.contract.source.rust.macros.mode == MacroExpansionMode::DenyUnreviewed;
    if !enforcing && context.contract.source.rust.macros.allow.is_empty() {
        return;
    }
    let allowances = &context.contract.source.rust.macros.allow;
    let allowed = AllowanceIndex::new(allowances);
    let mut used = BTreeSet::new();
    let mut rejected = BTreeSet::new();
    let mut opaque_attempted = BTreeSet::new();
    let mut opaque_used = BTreeSet::new();
    for file in context
        .source
        .files
        .iter()
        .filter(|file| !file.reachability.is_unreachable())
    {
        for expansion in &file.macro_expansions {
            if directly_inspected(expansion) {
                continue;
            }
            match review(context.source, context.resolved_cargo, expansion, &allowed) {
                MacroBindingResult::Bound {
                    allowances: matched,
                    ..
                } => {
                    mark(&mut used, allowances, &matched);
                }
                MacroBindingResult::Rejected {
                    attempted: matched,
                    reasons,
                } => {
                    mark(&mut rejected, allowances, &matched);
                    findings.push(unbound(file, expansion, &matched, &reasons));
                }
                MacroBindingResult::NoNameMatch if enforcing => {
                    findings.push(unreviewed(file, expansion));
                }
                MacroBindingResult::NoNameMatch => {}
            }
        }
        for input in &file.opaque_macro_inputs {
            let (matched, confidence) =
                match review(context.source, context.resolved_cargo, input, &allowed) {
                    MacroBindingResult::Bound {
                        allowances,
                        confidence,
                    } => (allowances, confidence),
                    MacroBindingResult::Rejected {
                        attempted: matched, ..
                    } => {
                        mark(&mut opaque_attempted, allowances, &matched);
                        continue;
                    }
                    MacroBindingResult::NoNameMatch => continue,
                };
            mark(&mut opaque_attempted, allowances, &matched);
            if matched
                .iter()
                .any(|allowance| allowance.inputs != MacroInputMode::Opaque)
            {
                findings.push(
                    Finding::error(
                        "RUST-MACRO-003",
                        "rust.macro-input",
                        "source",
                        format!("macro {} has unreviewed opaque input", input.name),
                    )
                    .at(&file.relative, input.span)
                    .with_analysis(confidence)
                    .with_help(
                        "use an understood Rust-expression macro form or explicitly set inputs = \"opaque\" after reviewing the DSL boundary",
                    ),
                );
                continue;
            }
            mark(&mut opaque_used, allowances, &matched);
        }
    }
    let mut attempted = used;
    attempted.extend(rejected);
    stale_allowances(
        allowances,
        &attempted,
        &opaque_attempted,
        &opaque_used,
        findings,
    );
}

fn stale_allowances(
    allowed: &[MacroExpansionAllow],
    attempted: &BTreeSet<usize>,
    opaque_attempted: &BTreeSet<usize>,
    opaque_used: &BTreeSet<usize>,
    findings: &mut FindingSink,
) {
    for (index, allowance) in allowed.iter().enumerate() {
        let name = &allowance.name;
        let provenance = provenance(allowance);
        if !attempted.contains(&index) {
            findings.push(
                Finding::error(
                    "RUST-MACRO-002",
                    "rust.macro-expansion",
                    "source",
                    format!(
                        "allowed macro expansion {name:?} from {provenance} matches no reachable invocation"
                    ),
                )
                .because(&allowance.reason)
                .with_help("remove stale macro expansion authority"),
            );
        }
        if allowance.inputs == MacroInputMode::Opaque
            && !opaque_attempted.contains(&index)
            && !opaque_used.contains(&index)
        {
            findings.push(
                Finding::error(
                    "RUST-MACRO-004",
                    "rust.macro-input",
                    "source",
                    format!("opaque input authority for macro {name:?} from {provenance} is stale"),
                )
                .because(&allowance.reason)
                .with_help("remove inputs = \"opaque\" until opaque input is actually required"),
            );
        }
    }
}

fn provenance(allowance: &MacroExpansionAllow) -> String {
    allowance.definition.as_ref().map_or_else(
        || {
            allowance.source.as_ref().map_or_else(
                || "unbound provenance".into(),
                zrail_core::CrateRootSource::identity,
            )
        },
        |definition| format!("definition {definition:?}"),
    )
}

fn mark(
    indices: &mut BTreeSet<usize>,
    all: &[MacroExpansionAllow],
    matched: &[&MacroExpansionAllow],
) {
    for allowance in matched {
        if let Some(index) = all
            .iter()
            .position(|candidate| std::ptr::eq(candidate, *allowance))
        {
            indices.insert(index);
        }
    }
}
