//! Allowed call owners require exact direct invocation evidence.

use zrail_core::{AnalysisQuality, Finding, FindingSink, OwnerContract};

use crate::source::{ObservedFact, RustFileFacts};

use super::path_matches;

pub(super) fn check(
    owner: &OwnerContract,
    file: &RustFileFacts,
    findings: &mut FindingSink,
) -> bool {
    let calls = matching(&file.calls, &owner.selector);
    let reference = matching(&file.paths, &owner.selector)
        .into_iter()
        .find(|reference| {
            reference.span.is_some()
                && !calls
                    .iter()
                    .any(|call| call.span == reference.span && call.name == reference.name)
        });
    if let Some(reference) = reference {
        reject(
            owner,
            file,
            reference,
            "is referenced outside a direct invocation",
            findings,
        );
        return true;
    }
    if let Some(call) = calls
        .iter()
        .find(|call| call.quality != AnalysisQuality::Exact)
    {
        reject(
            owner,
            file,
            call,
            "cannot be resolved to an exact direct invocation",
            findings,
        );
        return true;
    }
    !calls.is_empty()
}

fn matching<'a>(facts: &'a [ObservedFact], selector: &str) -> Vec<&'a ObservedFact> {
    facts
        .iter()
        .filter(|fact| path_matches(selector, fact))
        .collect()
}

fn reject(
    owner: &OwnerContract,
    file: &RustFileFacts,
    fact: &ObservedFact,
    detail: &str,
    findings: &mut FindingSink,
) {
    findings.push(
        Finding::error(
            "OWN-005",
            &owner.name,
            "ownership",
            format!("owned call {} {detail}", owner.selector),
        )
        .at(&file.relative, fact.span)
        .because(&owner.reason)
        .with_analysis(fact.quality)
        .with_help("invoke the owned call directly so zrail can verify its authority boundary"),
    );
}
