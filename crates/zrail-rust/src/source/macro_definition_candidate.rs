//! Repository macro candidates retain exact names, origins, and definition sites.

use std::collections::BTreeSet;

use zrail_core::AnalysisQuality;

use super::{
    MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin, ObservedFact,
    macro_definitions::DefinitionSite,
};

pub(super) fn repository_candidate(
    expansion: &MacroExpansionFact,
    policy_name: &str,
    site: DefinitionSite,
) -> MacroCandidate {
    MacroCandidate {
        observation: ObservedFact {
            name: policy_name.into(),
            written: None,
            implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
            canonical: Vec::new(),
            span: expansion.span,
            quality: AnalysisQuality::Exact,
            guard: expansion.guard.clone(),
            lexical_scope: expansion.lexical_scope.clone(),
            namespace: super::FactNamespace::Unknown,
            generic_shadow: None,
        },
        origins: vec![MacroOrigin::Repository {
            package: site.package,
            directory: site.directory,
        }],
        derivation: MacroDerivation::LocalDefinition,
        written_alias: false,
        definition: Some(site.file),
        definition_name: Some(site.name),
        definition_sha256: Some(site.sha256),
    }
}

pub(super) fn candidate_order(left: &MacroCandidate, right: &MacroCandidate) -> std::cmp::Ordering {
    left.observation
        .name
        .cmp(&right.observation.name)
        .then(left.definition.cmp(&right.definition))
        .then(left.definition_name.cmp(&right.definition_name))
        .then(left.definition_sha256.cmp(&right.definition_sha256))
        .then(left.origins.cmp(&right.origins))
        .then(left.derivation.cmp(&right.derivation))
}

pub(super) fn local_policy_name(expansion: &MacroExpansionFact) -> String {
    let names = expansion
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.derivation == MacroDerivation::LocalDefinition
                || super::macro_visibility::repository_path(&candidate.observation.name)
        })
        .map(|candidate| candidate.observation.name.as_str())
        .collect::<BTreeSet<_>>();
    if names.len() == 1 {
        names.into_iter().next().unwrap_or(&expansion.name).into()
    } else {
        expansion.name.clone()
    }
}

pub(super) fn discard_file_wide_definition_guess(expansion: &mut MacroExpansionFact) {
    let written = expansion.name.clone();
    for candidate in &mut expansion.candidates {
        if candidate.derivation != MacroDerivation::LocalDefinition
            || candidate.observation.name != written
        {
            continue;
        }
        candidate.derivation = MacroDerivation::Written;
        candidate.observation.canonical.clear();
        candidate.observation.quality = AnalysisQuality::Exact;
        for origin in &mut candidate.origins {
            if matches!(origin, MacroOrigin::Pending { .. }) {
                *origin = MacroOrigin::Pending {
                    local_module: false,
                };
            }
        }
    }
}

pub(super) fn add_include_scope_uncertainty(expansion: &mut MacroExpansionFact) {
    if !expansion
        .candidates
        .iter()
        .any(|candidate| candidate.derivation == MacroDerivation::Written)
    {
        return;
    }
    let mut observation = expansion.observation.clone();
    observation.canonical = vec![expansion.name.clone()];
    observation.quality = AnalysisQuality::Unresolved;
    expansion.candidates.push(MacroCandidate::unresolved(
        observation,
        MacroDerivation::Written,
    ));
}
