//! Unexpanded Rust is an explicit, content-bound, reasoned trust boundary.

mod source;

use std::collections::{BTreeMap, BTreeSet};

use zrail_core::{
    AnalysisQuality, Finding, FindingSink, MacroExpansionAllow, MacroExpansionMode, MacroInputMode,
};

use crate::source::{ObservedFact, Reachability, RustFileFacts};

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
            let matched = reviewed_for_file(context, file, expansion, &allowed);
            if matched.is_empty() {
                findings.push(unreviewed(file, expansion));
            } else {
                used.extend(matched);
            }
        }
        for input in &file.opaque_macro_inputs {
            let matched = reviewed_for_file(context, file, input, &allowed);
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
    validate_local_bindings(context, &allowed, findings);
}

fn unreviewed(file: &crate::source::RustFileFacts, expansion: &ObservedFact) -> Finding {
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

fn validate_local_bindings(
    context: &RuleContext<'_>,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
    findings: &mut FindingSink,
) {
    for allowance in allowed
        .values()
        .filter(|allowance| allowance.definition.is_some())
    {
        let path = allowance.definition.as_deref().unwrap_or_default();
        let leaf = allowance
            .name
            .rsplit("::")
            .next()
            .unwrap_or(&allowance.name);
        let bound = context
            .source
            .files
            .iter()
            .find(|file| file.relative == path)
            .map_or(0, |file| {
                file.macro_definitions
                    .iter()
                    .filter(|definition| definition.name == leaf)
                    .count()
            });
        let total = context
            .source
            .files
            .iter()
            .filter(|file| file.reachability != Reachability::Unreachable)
            .flat_map(|file| &file.macro_definitions)
            .filter(|definition| definition.name == leaf)
            .count();
        if bound != 1 || total != 1 {
            findings.push(
                Finding::error(
                    "RUST-MACRO-005",
                    "rust.macro-definition",
                    "source",
                    format!(
                        "local macro allowance {:?} resolves to {bound} definitions in {path:?} and {total} reachable definitions repository-wide",
                        allowance.name,
                    ),
                )
                .because(&allowance.reason)
                .with_help("bind the allowance to one exact local macro_rules! definition"),
            );
        }
    }
}

fn reviewed_names<'a>(
    expansion: &'a ObservedFact,
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

fn reviewed_for_file<'a>(
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    expansion: &'a ObservedFact,
    allowed: &BTreeMap<&str, &MacroExpansionAllow>,
) -> Vec<&'a str> {
    let names = reviewed_names(expansion, allowed);
    if names
        .iter()
        .all(|name| source::bound(context, file, allowed[name]))
    {
        names
    } else {
        Vec::new()
    }
}

fn directly_inspected(expansion: &ObservedFact) -> bool {
    if expansion.quality != AnalysisQuality::Exact {
        return false;
    }
    expansion.policy_names().all(|name| {
        let Some(leaf) = (!name.contains("::")).then_some(name) else {
            return false;
        };
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
