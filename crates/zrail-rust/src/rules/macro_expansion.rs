//! Unexpanded Rust is an explicit, content-bound, reasoned trust boundary.

mod bindings;
mod diagnostics;
mod failure;
mod review;
mod source;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    AnalysisQuality, Finding, FindingSink, MacroExpansionAllow, MacroExpansionMode, MacroInputMode,
};

use crate::source::MacroExpansionFact;

use super::RuleContext;

use diagnostics::{unbound, unreviewed};
#[cfg(test)]
use review::candidate_names;
#[cfg(test)]
use review::review_without_definitions;
use review::{MacroBindingResult, review};

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    if context.contract.source.rust.macros.mode == MacroExpansionMode::Allow {
        return;
    }
    let allowed = context
        .contract
        .source
        .rust
        .macros
        .allow
        .iter()
        .map(|allowed| (allowed.name.as_str(), allowed))
        .collect::<BTreeMap<_, _>>();
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
            match review(context, expansion, &allowed) {
                MacroBindingResult::Bound { allowances, .. } => {
                    used.extend(allowances);
                }
                MacroBindingResult::Rejected {
                    attempted: matched,
                    reasons,
                } => {
                    rejected.extend(matched.iter().copied());
                    findings.push(unbound(file, expansion, &matched, &reasons));
                }
                MacroBindingResult::NoNameMatch => findings.push(unreviewed(file, expansion)),
            }
        }
        for input in &file.opaque_macro_inputs {
            let (matched, confidence) = match review(context, input, &allowed) {
                MacroBindingResult::Bound {
                    allowances,
                    confidence,
                } => (allowances, confidence),
                MacroBindingResult::Rejected {
                    attempted: matched, ..
                } => {
                    opaque_attempted.extend(matched);
                    continue;
                }
                MacroBindingResult::NoNameMatch => continue,
            };
            opaque_attempted.extend(matched.iter().copied());
            if matched
                .iter()
                .any(|name| allowed[*name].inputs != MacroInputMode::Opaque)
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
            opaque_used.extend(matched);
        }
    }
    let mut attempted = used;
    attempted.extend(rejected);
    stale_allowances(
        &allowed,
        &attempted,
        &opaque_attempted,
        &opaque_used,
        findings,
    );
}

fn stale_allowances(
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
    attempted: &BTreeSet<&str>,
    opaque_attempted: &BTreeSet<&str>,
    opaque_used: &BTreeSet<&str>,
    findings: &mut FindingSink,
) {
    for (name, allowance) in allowed {
        if !attempted.contains(name) {
            findings.push(
                Finding::error(
                    "RUST-MACRO-002",
                    "rust.macro-expansion",
                    "source",
                    format!("allowed macro expansion {name:?} matches no reachable invocation"),
                )
                .because(&allowance.reason)
                .with_help("remove stale macro expansion authority"),
            );
        }
        if allowance.inputs == MacroInputMode::Opaque
            && !opaque_attempted.contains(name)
            && !opaque_used.contains(name)
        {
            findings.push(
                Finding::error(
                    "RUST-MACRO-004",
                    "rust.macro-input",
                    "source",
                    format!("opaque input authority for macro {name:?} is stale"),
                )
                .because(&allowance.reason)
                .with_help("remove inputs = \"opaque\" until opaque input is actually required"),
            );
        }
    }
}

fn directly_inspected(expansion: &MacroExpansionFact) -> bool {
    if expansion.quality != AnalysisQuality::Exact || !expansion.is_compiler_builtin() {
        return false;
    }
    expansion
        .candidates
        .iter()
        .flat_map(crate::source::MacroCandidate::policy_names)
        .all(|name| {
            let leaf = name.rsplit("::").next().unwrap_or(name);
            matches!(
                leaf,
                "cfg"
                    | "column"
                    | "concat"
                    | "concat_bytes"
                    | "env"
                    | "file"
                    | "include"
                    | "include_bytes"
                    | "include_str"
                    | "line"
                    | "module_path"
                    | "option_env"
                    | "stringify"
            )
        })
}

#[cfg(test)]
#[path = "macro_expansion_test.rs"]
mod macro_expansion_test;
