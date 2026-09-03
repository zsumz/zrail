//! Same-name macro authorities bind independently by exact implementation source.

use zrail_core::{
    AnalysisQuality, CrateRootSource, MacroBindingMode, MacroExpansionAllow,
    MacroExpansionBindings, MacroInputMode,
};

use crate::{
    cargo::DependencySource,
    source::{
        FactNamespace, MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin,
        ObservedFact, SourceIndex, SyntaxGuard,
    },
};

use super::super::review::{MacroBindingResult, review};
use super::AllowanceIndex;

#[test]
fn exact_origin_selects_the_matching_same_name_allowance() {
    let one = allowance("=1.0.0");
    let two = allowance("=2.0.0");
    let allowed = AllowanceIndex::new([&one, &two]);

    for requirement in ["=1.0.0", "=2.0.0"] {
        let expansion = expansion(requirement);
        let MacroBindingResult::Bound { allowances, .. } =
            review(&SourceIndex::default(), None, &expansion, &allowed)
        else {
            panic!("{requirement} origin must bind its reviewed authority");
        };
        assert_eq!(allowances.len(), 1);
        assert_eq!(allowances[0].source, Some(registry(requirement)));
    }
}

fn allowance(requirement: &str) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: "derive::Model".into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: zrail_core::MacroAsyncSyntax::Opaque,
        duplication_effect: zrail_core::MacroDuplicationEffect::Opaque,
        source_operations: zrail_core::MacroSourceOperations::Opaque,
        field_mutation: zrail_core::MacroFieldMutation::Opaque,
        definition: None,
        source: Some(registry(requirement)),
        reason: "The exact macro implementation was reviewed.".into(),
    }
}

fn registry(requirement: &str) -> CrateRootSource {
    CrateRootSource::Registry {
        registry: None,
        index: None,
        requirement: requirement.into(),
    }
}

fn expansion(requirement: &str) -> MacroExpansionFact {
    let observed = fact();
    MacroExpansionFact::with_candidates(
        observed.clone(),
        vec![MacroCandidate {
            observation: observed,
            origins: vec![MacroOrigin::External {
                package: "derive".into(),
                source: DependencySource::Registry {
                    registry: None,
                    index: None,
                    requirement: requirement.into(),
                },
            }],
            derivation: MacroDerivation::DependencyRoot,
            written_alias: false,
            definition: None,
            definition_name: None,
            definition_sha256: None,
        }],
    )
}

fn fact() -> ObservedFact {
    ObservedFact {
        name: "derive::Model".into(),
        written: None,
        implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
        guard: SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Unknown,
        generic_shadow: None,
        associated_candidates: Vec::new(),
        inherits_parent_context: true,
    }
}
