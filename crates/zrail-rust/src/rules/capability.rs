//! Exact symbol scopes and semantic effect profiles.

mod compile;
mod effects;
mod ownership;
mod ownership_call;
mod ownership_operation;
mod ownership_operation_macros;
mod profiles;
mod syntax;

use zrail_core::{AnalysisQuality, Finding, FindingSink, glob_matches};

use crate::source::ObservedFact;

use super::RuleContext;

pub(super) use effects::tokens as effect_tokens;
pub(crate) use ownership::matching_capability as matching_capability_owner;
pub(crate) use ownership_call::{
    CallOwnerEvidenceKind, matching_evidence as matching_call_owner_evidence,
};
pub(crate) use ownership_operation::matching_operations as matching_operation_owner_operations;
pub(crate) use profiles::assigned_profiles;
pub(crate) use syntax::syntax_name as async_syntax_name;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    check_exact_scopes(context, findings);
    profiles::check(context, findings);
    compile::check_paths(context, findings);
    ownership::check(context, findings);
}

fn check_exact_scopes(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for scope in &context.contract.scopes {
        let mut stale = false;
        for pattern in &scope.include {
            if !context
                .source
                .files
                .iter()
                .any(|file| glob_matches(pattern, &file.relative))
            {
                stale = true;
                findings.push(
                    Finding::error(
                        "CAP-002",
                        &scope.name,
                        "capability",
                        format!("scope include {pattern:?} matches no Rust source"),
                    )
                    .because(&scope.reason),
                );
            }
        }
        for pattern in &scope.exclude {
            if !context
                .source
                .files
                .iter()
                .any(|file| glob_matches(pattern, &file.relative))
            {
                stale = true;
                findings.push(
                    Finding::error(
                        "CAP-003",
                        &scope.name,
                        "capability",
                        format!("scope exclusion {pattern:?} matches no Rust source"),
                    )
                    .because(&scope.reason),
                );
            }
        }
        if stale
            || !context
                .source
                .files
                .iter()
                .any(|file| matches_scope(&file.relative, &scope.include, &scope.exclude))
        {
            if !stale {
                findings.push(
                    Finding::error(
                        "CAP-002",
                        &scope.name,
                        "capability",
                        "capability scope matches no Rust source after exclusions",
                    )
                    .because(&scope.reason),
                );
            }
            continue;
        }
        for file in context
            .source
            .files
            .iter()
            .filter(|file| matches_scope(&file.relative, &scope.include, &scope.exclude))
        {
            for denied in &scope.symbols.deny {
                for path in file
                    .paths
                    .iter()
                    .chain(&file.macros)
                    .filter(|path| path_matches(denied, path))
                {
                    findings.push(
                        Finding::error(
                            "CAP-001",
                            &scope.name,
                            "capability",
                            format!("source reaches denied symbol {}", path.name),
                        )
                        .at(&file.relative, path.span)
                        .because(&scope.reason)
                        .with_analysis(path.quality)
                        .with_help("move the effect to its owning adapter and pass facts inward"),
                    );
                }
            }
        }
    }
}

fn matches_scope(path: &str, include: &[String], exclude: &[String]) -> bool {
    include.iter().any(|pattern| glob_matches(pattern, path))
        && !exclude.iter().any(|pattern| glob_matches(pattern, path))
}

pub(super) fn path_matches(denied: &str, observed: &ObservedFact) -> bool {
    let denied = normalized_path(denied);
    observed.policy_names().any(|name| {
        let name = normalized_path(name);
        name == denied
            || name.starts_with(&format!("{denied}::"))
            || (observed.quality != AnalysisQuality::Exact
                && denied.starts_with(&format!("{name}::")))
    })
}

pub(super) fn normalized_path(path: &str) -> String {
    let path = path.strip_prefix("::").unwrap_or(path);
    let path = path.strip_prefix("crate::").unwrap_or(path);
    path.split("::")
        .map(|segment| segment.strip_prefix("r#").unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
#[path = "capability_test.rs"]
mod capability_test;
