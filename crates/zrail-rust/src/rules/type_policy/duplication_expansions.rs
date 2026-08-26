//! A Clone/Copy-closed type closes every impl-producing macro in its compilation world.

use std::collections::BTreeSet;

use zrail_core::{Finding, FindingSink, RustTypeContract};

use crate::source::{RustFileFacts, TypeDeclarationFact};

use super::{RuleContext, identity};

pub(super) fn check(
    context: &RuleContext<'_>,
    policy: &RustTypeContract,
    declaration_file: &RustFileFacts,
    declaration: &TypeDeclarationFact,
    findings: &mut FindingSink,
) {
    let declaration_domains = identity::domain_identities(
        context,
        declaration_file,
        &declaration.guard,
        policy.reachability,
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let packages = declaration_file.packages.iter().collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();
    for file in context.source.files.iter().filter(|file| {
        file.packages
            .iter()
            .any(|package| packages.contains(package))
    }) {
        for fact in file.item_macros.iter().chain(&file.opaque_binding_macros) {
            let Some(span) = fact.span else {
                continue;
            };
            if !identity::applies(context, file, &fact.guard, policy.reachability) {
                continue;
            }
            let overlaps =
                identity::domain_identities(context, file, &fact.guard, policy.reachability)
                    .into_iter()
                    .any(|domain| declaration_domains.contains(&domain));
            if !overlaps || !emitted.insert((file.relative.clone(), span)) {
                continue;
            }
            let closed = file.macro_expansions.iter().any(|expansion| {
                expansion.span == Some(span)
                    && crate::rules::closes_type_duplication(
                        context.contract,
                        context.source,
                        context.resolved_cargo,
                        expansion,
                    )
            });
            if !closed {
                findings.push(
                    Finding::error(
                        "RUST-TYPE-005",
                        &policy.name,
                        "type-policy",
                        format!(
                            "macro expansion {} may add Clone/Copy implementations for closed type {}",
                            fact.name, policy.identity
                        ),
                    )
                    .at(&file.relative, Some(span))
                    .because(&policy.reason)
                    .with_analysis(fact.quality)
                    .with_help(
                        "bind the exact macro provenance and attest duplication_effect = \"none\"",
                    ),
                );
            }
        }
    }
}
