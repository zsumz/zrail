//! Unexpanded Rust is an explicit, content-bound, reasoned trust boundary.

mod bindings;
mod source;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    AnalysisQuality, Finding, FindingSink, MacroExpansionAllow, MacroExpansionMode, MacroInputMode,
};

use crate::source::{MacroExpansionFact, Reachability};

use super::RuleContext;

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
            let matched = reviewed(expansion, &allowed);
            if matched.is_empty() {
                findings.push(unreviewed(file, expansion));
            } else {
                used.extend(matched);
            }
        }
        for input in &file.opaque_macro_inputs {
            let matched = reviewed(input, &allowed);
            if matched.is_empty()
                || matched
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
            } else {
                opaque_used.extend(matched);
            }
        }
    }
    stale_allowances(&allowed, &used, &opaque_used, findings);
    bindings::validate(context, &allowed, findings);
}

fn unreviewed(file: &crate::source::RustFileFacts, expansion: &MacroExpansionFact) -> Finding {
    Finding::error(
        "RUST-MACRO-001",
        "rust.macro-expansion",
        "source",
        format!("source invokes unreviewed macro expansion {}", expansion.name),
    )
    .at(&file.relative, expansion.span)
    .with_analysis(expansion.quality)
    .with_help(
        "remove the macro or add a reasoned source.rust.macros.allow entry after reviewing its expansion boundary",
    )
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

fn reviewed_names<'a>(
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> Vec<&'a str> {
    if expansion.quality == AnalysisQuality::Unresolved {
        return Vec::new();
    }
    let names = expansion.policy_names().collect::<Vec<_>>();
    if names.iter().all(|name| allowed.contains_key(name)) {
        names
    } else {
        Vec::new()
    }
}

fn reviewed<'a>(
    expansion: &'a MacroExpansionFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> Vec<&'a str> {
    let names = reviewed_names(expansion, allowed);
    if names
        .iter()
        .all(|name| source::bound(expansion, allowed[name]))
    {
        names
    } else {
        Vec::new()
    }
}

fn directly_inspected(expansion: &MacroExpansionFact) -> bool {
    if expansion.quality != AnalysisQuality::Exact || !expansion.is_compiler_builtin() {
        return false;
    }
    expansion.policy_names().all(|name| {
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
