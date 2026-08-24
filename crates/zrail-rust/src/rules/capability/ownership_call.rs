//! Allowed call owners require exact direct invocation evidence.

use zrail_core::{AnalysisQuality, Finding, FindingSink, OwnerContract};

use crate::source::{ObservedFact, RustFileFacts};

use super::{ownership::fact_applies, path_matches};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallOwnerEvidenceKind {
    DirectCall,
    Reference,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CallOwnerEvidence<'a> {
    pub(crate) kind: CallOwnerEvidenceKind,
    pub(crate) fact: &'a ObservedFact,
}

pub(super) fn check(
    owner: &OwnerContract,
    file: &RustFileFacts,
    findings: &mut FindingSink,
) -> bool {
    let evidence = matching_evidence(owner, file);
    let reference = evidence
        .into_iter()
        .find(|evidence| evidence.kind == CallOwnerEvidenceKind::Reference);
    if let Some(reference) = reference {
        reject(
            owner,
            file,
            reference.fact,
            "is referenced outside a direct invocation",
            findings,
        );
        return true;
    }
    let calls = matching_evidence(owner, file);
    if let Some(call) = calls.iter().find(|evidence| {
        evidence.kind == CallOwnerEvidenceKind::DirectCall
            && evidence.fact.quality != AnalysisQuality::Exact
    }) {
        reject(
            owner,
            file,
            call.fact,
            "cannot be resolved to an exact direct invocation",
            findings,
        );
        return true;
    }
    calls
        .iter()
        .any(|evidence| evidence.kind == CallOwnerEvidenceKind::DirectCall)
}

pub(crate) fn matching_evidence<'a>(
    owner: &OwnerContract,
    file: &'a RustFileFacts,
) -> Vec<CallOwnerEvidence<'a>> {
    let calls = matching(owner, file, &file.calls);
    let mut evidence = calls
        .iter()
        .map(|fact| CallOwnerEvidence {
            kind: CallOwnerEvidenceKind::DirectCall,
            fact,
        })
        .collect::<Vec<_>>();
    evidence.extend(
        matching(owner, file, &file.paths)
            .into_iter()
            .filter(|reference| {
                reference.span.is_some()
                    && !calls
                        .iter()
                        .any(|call| call.span == reference.span && call.name == reference.name)
            })
            .map(|fact| CallOwnerEvidence {
                kind: CallOwnerEvidenceKind::Reference,
                fact,
            }),
    );
    evidence
}

fn matching<'a>(
    owner: &OwnerContract,
    file: &RustFileFacts,
    facts: &'a [ObservedFact],
) -> Vec<&'a ObservedFact> {
    facts
        .iter()
        .filter(|fact| fact_applies(owner, file, fact) && path_matches(&owner.selector, fact))
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
