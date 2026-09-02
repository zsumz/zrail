//! Each glob-derived invocation candidate is checked against its target export set.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::super::{
    GuardAvailability, MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin,
    ObservedFact,
};
use super::{ExportedMacro, MacroExports, MountedImport};

const MAX_UNKNOWN_REASONS: usize = 4;

impl MacroExports {
    pub(in crate::source) fn apply(
        &self,
        expansion: &mut MacroExpansionFact,
        file: &str,
        syntax: super::super::SourceSyntax,
    ) {
        if expansion
            .candidates
            .iter()
            .all(|candidate| import_derived(candidate.derivation))
        {
            expansion.candidates.push(MacroCandidate::pending(
                expansion.observation.clone(),
                false,
                MacroDerivation::Written,
            ));
        }
        let contexts = self
            .contexts
            .get(&(file.into(), syntax, expansion.lexical_scope.clone()));
        let mut candidates = Vec::new();
        let mut resolved_contexts = BTreeSet::new();
        let expansion_guard = expansion.guard.clone();
        let active_contexts = contexts
            .into_iter()
            .flat_map(|contexts| contexts.iter())
            .filter(|context| {
                context
                    .guard
                    .combine(&expansion_guard)
                    .availability_in_domain(&context.module.domain)
                    .is_available()
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        for candidate in expansion.candidates.drain(..) {
            if !import_derived(candidate.derivation) {
                candidates.push(candidate);
                continue;
            }
            let Some(contexts) = contexts else {
                candidates.push(unknown_candidate(
                    candidate,
                    BTreeSet::from(["logical invocation mount is unavailable".into()]),
                ));
                continue;
            };
            let mut unknown = BTreeSet::new();
            for context in contexts {
                if !context
                    .guard
                    .combine(&candidate.observation.guard)
                    .availability_in_domain(&context.module.domain)
                    .is_available()
                {
                    continue;
                }
                let resolved = self.resolve_import(&candidate, context);
                if resolved.resolved {
                    resolved_contexts.insert(context.clone());
                }
                candidates.extend(resolved.candidates);
                unknown.extend(resolved.unknown);
            }
            if !unknown.is_empty() {
                candidates.push(unknown_candidate(candidate, unknown));
            }
        }
        for context in &active_contexts {
            for import in &context.inherited_imports {
                let candidate = inherited_candidate(expansion, import);
                let resolved = self.resolve_import(&candidate, context);
                if resolved.resolved {
                    resolved_contexts.insert(context.clone());
                }
                candidates.extend(resolved.candidates);
                if !resolved.unknown.is_empty() {
                    candidates.push(unknown_candidate(candidate, resolved.unknown));
                }
            }
        }
        candidates.retain(|candidate| {
            candidate.derivation != MacroDerivation::Written
                || active_contexts.is_empty()
                || resolved_contexts != active_contexts
        });
        candidates.sort_by(candidate_order);
        candidates.dedup();
        expansion.candidates = candidates;
        expansion.refresh_quality();
    }
}

fn import_derived(derivation: MacroDerivation) -> bool {
    matches!(
        derivation,
        MacroDerivation::ExactImport | MacroDerivation::GlobImport | MacroDerivation::ReExport
    )
}

fn inherited_candidate(expansion: &MacroExpansionFact, import: &MountedImport) -> MacroCandidate {
    let mut observation: ObservedFact = expansion.observation.clone();
    observation.name.clone_from(&import.target);
    observation.canonical.clear();
    observation.guard = observation.guard.combine(&import.guard);
    observation.quality = observation.quality.max(import.quality);
    MacroCandidate::pending(observation, false, import.derivation)
}

pub(super) fn resolved_candidate(
    candidate: &MacroCandidate,
    exported: &ExportedMacro,
    availability: GuardAvailability,
    module: &super::LogicalModule,
    exports: &MacroExports,
) -> MacroCandidate {
    let mut observation = candidate.observation.clone();
    if let Some(authority_name) = &exported.authority_name {
        observation.name.clone_from(authority_name);
    }
    observation.quality = exported
        .quality
        .max(if availability == GuardAvailability::Possible {
            AnalysisQuality::Conservative
        } else {
            AnalysisQuality::Exact
        });
    observation.guard = observation.guard.combine(&exported.guard);
    MacroCandidate {
        observation,
        origins: if exported.origins.is_empty() {
            exports.repository_origin(module)
        } else {
            exported.origins.clone()
        },
        derivation: candidate.derivation,
        written_alias: candidate.written_alias,
        definition: exported.definition.clone(),
        definition_name: exported.definition_name.clone(),
        definition_sha256: exported.definition_sha256.clone(),
    }
}

fn unknown_candidate(mut candidate: MacroCandidate, reasons: BTreeSet<String>) -> MacroCandidate {
    candidate.observation.canonical.clear();
    candidate.observation.quality = AnalysisQuality::Unresolved;
    candidate.origins = vec![MacroOrigin::UnknownExportSet {
        reason: summarized_reasons(reasons),
    }];
    candidate.definition = None;
    candidate.definition_name = None;
    candidate.definition_sha256 = None;
    candidate
}

fn summarized_reasons(reasons: BTreeSet<String>) -> String {
    let count = reasons.len();
    let mut selected = reasons
        .into_iter()
        .take(MAX_UNKNOWN_REASONS)
        .collect::<Vec<_>>();
    if count > MAX_UNKNOWN_REASONS {
        selected.push(format!(
            "{} additional unknown export set(s)",
            count - MAX_UNKNOWN_REASONS
        ));
    }
    selected.join("; ")
}

fn candidate_order(left: &MacroCandidate, right: &MacroCandidate) -> std::cmp::Ordering {
    left.observation
        .name
        .cmp(&right.observation.name)
        .then(left.observation.quality.cmp(&right.observation.quality))
        .then(left.origins.cmp(&right.origins))
        .then(left.derivation.cmp(&right.derivation))
        .then(left.definition.cmp(&right.definition))
        .then(left.definition_name.cmp(&right.definition_name))
        .then(left.definition_sha256.cmp(&right.definition_sha256))
}
