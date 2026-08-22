//! Unexpanded Rust is an explicit, content-bound, reasoned trust boundary.

mod bindings;
mod review;
mod source;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    AnalysisQuality, Finding, FindingSink, MacroExpansionAllow, MacroExpansionMode, MacroInputMode,
};

use crate::source::{MacroExpansionFact, Reachability};

use super::RuleContext;

#[cfg(test)]
use review::candidate_names;
use review::{Review, review};

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
    let mut opaque_used = BTreeSet::new();
    for file in context
        .source
        .files
        .iter()
        .filter(|file| file.reachability != Reachability::Unreachable)
    {
        for expansion in &file.macro_expansions {
            if directly_inspected(expansion) {
                continue;
            }
            match review(expansion, &allowed) {
                Review::Allowed(matched) => used.extend(matched),
                Review::Unbound => findings.push(unbound(file, expansion)),
                Review::Unreviewed => findings.push(unreviewed(file, expansion)),
            }
        }
        for input in &file.opaque_macro_inputs {
            let Review::Allowed(matched) = review(input, &allowed) else {
                continue;
            };
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
                    .with_analysis(input.quality)
                    .with_help(
                        "use an understood Rust-expression macro form or explicitly set inputs = \"opaque\" after reviewing the DSL boundary",
                    ),
                );
                continue;
            }
            opaque_used.extend(matched);
        }
    }
    stale_allowances(&allowed, &used, &opaque_used, findings);
    bindings::validate(context, &allowed, findings);
}

fn unbound(file: &crate::source::RustFileFacts, expansion: &MacroExpansionFact) -> Finding {
    Finding::error(
        "RUST-MACRO-006",
        "rust.macro-binding",
        "source",
        format!(
            "reviewed macro allowance could not bind invocation {}",
            expansion.name
        ),
    )
    .at(&file.relative, expansion.span)
    .with_analysis(expansion.quality)
    .with_help(
        "resolve the macro origin or use binding = \"conservative\" on a name-only allowance after reviewing the unresolved invocation",
    )
}

fn unreviewed(file: &crate::source::RustFileFacts, expansion: &MacroExpansionFact) -> Finding {
    let preferred = expansion.preferred_policy_name();
    let message = preferred
        .filter(|name| *name != expansion.name)
        .map_or_else(
            || {
                format!(
                    "source invokes unreviewed macro expansion {}",
                    expansion.name
                )
            },
            |name| {
                format!(
                    "source invokes unreviewed macro expansion {} (preferred policy name {name})",
                    expansion.name
                )
            },
        );
    let help = preferred.map_or_else(
        || {
            "remove the macro or add a reasoned source.rust.macros.allow entry after reviewing its expansion boundary".into()
        },
        |name| {
            format!(
                "remove the macro or add source.rust.macros.allow name = {name:?} after reviewing its expansion boundary"
            )
        },
    );
    Finding::error("RUST-MACRO-001", "rust.macro-expansion", "source", message)
        .at(&file.relative, expansion.span)
        .with_analysis(expansion.quality)
        .with_help(help)
}

fn stale_allowances(
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
    used: &BTreeSet<&str>,
    opaque_used: &BTreeSet<&str>,
    findings: &mut FindingSink,
) {
    for (name, allowance) in allowed {
        if !used.contains(name) {
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
        if allowance.inputs == MacroInputMode::Opaque && !opaque_used.contains(name) {
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
