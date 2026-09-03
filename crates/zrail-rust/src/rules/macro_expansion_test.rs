//! Directly inspected compiler macros cannot be confused with local definitions.

use zrail_core::{
    AnalysisQuality, MacroBindingMode, MacroExpansionAllow, MacroExpansionBindings, MacroInputMode,
};

use crate::source::{
    MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin, ObservedFact,
};

use super::{
    MacroBindingResult, allowances::AllowanceIndex, candidate_names, directly_inspected,
    review_without_definitions,
};

#[test]
fn local_definitions_shadow_intrinsic_shortcuts() {
    let include = compiler("include");
    let mut local_include = compiler("include");
    local_include.quality = AnalysisQuality::Unresolved;

    assert!(directly_inspected(&include));
    assert!(!directly_inspected(&local_include));
    assert!(directly_inspected(&compiler("std::include")));
    assert!(directly_inspected(&compiler("core::concat")));

    let mut aliased = compiler("std::env");
    aliased.name = "read_env".into();
    aliased.candidates[0].written_alias = true;
    assert!(directly_inspected(&aliased));
}

#[test]
fn arbitrary_expression_macros_are_never_assumed_inspected() {
    for name in ["assert", "format", "matches", "vec", "tokio::select"] {
        assert!(!directly_inspected(&compiler(name)));
    }
}

#[test]
fn every_conservative_canonical_identity_requires_review() {
    let mut expansion = expansion("runtime::select");
    expansion.candidates[0].observation.canonical =
        vec!["async_std::select".into(), "tokio::select".into()];
    expansion.candidates[0].observation.quality = AnalysisQuality::Conservative;
    let async_std = allowance("async_std::select");
    let tokio = allowance("tokio::select");
    let partial = AllowanceIndex::new([&tokio]);
    let complete = AllowanceIndex::new([&async_std, &tokio]);

    assert!(candidate_names(&expansion, &expansion.candidates[0], &partial).is_none());
    assert_eq!(
        candidate_names(&expansion, &expansion.candidates[0], &complete).map(|names| names.len()),
        Some(2)
    );
}

#[test]
fn exact_allowance_cannot_bind_an_unresolved_written_macro() {
    let reviewed = allowance("reviewed");
    let allowed = AllowanceIndex::new([&reviewed]);
    let local = unresolved("reviewed");

    assert!(matches!(
        review_without_definitions(&local, &allowed),
        MacroBindingResult::Rejected { .. }
    ));
}

#[test]
fn conservative_bare_allowance_binds_only_the_written_name() {
    let mut reviewed = allowance("reviewed");
    reviewed.binding = MacroBindingMode::Conservative;
    let allowed = AllowanceIndex::new([&reviewed]);
    assert!(matches!(
        review_without_definitions(&unresolved("reviewed"), &allowed),
        MacroBindingResult::Bound { .. }
    ));

    let qualified = allowance("support::reviewed");
    let qualified_allowed = AllowanceIndex::new([&qualified]);
    assert!(matches!(
        review_without_definitions(&unresolved("reviewed"), &qualified_allowed),
        MacroBindingResult::NoNameMatch
    ));
}

#[test]
fn ambiguous_glob_candidates_all_require_allowances() {
    let expansion = MacroExpansionFact::with_candidates(
        fact("reviewed"),
        vec![
            repository("one::reviewed", MacroDerivation::GlobImport),
            repository("two::reviewed", MacroDerivation::GlobImport),
        ],
    );
    let one = allowance("one::reviewed");
    let two = allowance("two::reviewed");
    let partial = AllowanceIndex::new([&one]);
    let complete = AllowanceIndex::new([&one, &two]);

    assert!(matches!(
        review_without_definitions(&expansion, &partial),
        MacroBindingResult::Rejected { .. }
    ));
    assert!(matches!(
        review_without_definitions(&expansion, &complete),
        MacroBindingResult::Bound { .. }
    ));
}

fn allowance(name: &str) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: name.into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::Opaque,
        async_syntax: zrail_core::MacroAsyncSyntax::Opaque,
        duplication_effect: zrail_core::MacroDuplicationEffect::Opaque,
        source_operations: zrail_core::MacroSourceOperations::Opaque,
        field_mutation: zrail_core::MacroFieldMutation::Opaque,
        definition: name.starts_with("local::").then(|| "src/lib.rs".into()),
        source: None,
        reason: "reviewed".into(),
    }
}

fn fact(name: &str) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
        canonical: Vec::new(),
        span: None,
        quality: AnalysisQuality::Exact,
        guard: crate::source::SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: crate::source::FactNamespace::Unknown,
        generic_shadow: None,
        associated_candidates: Vec::new(),
        inherits_parent_context: true,
    }
}

fn expansion(name: &str) -> MacroExpansionFact {
    MacroExpansionFact::with_candidates(
        fact(name),
        vec![repository(name, MacroDerivation::Written)],
    )
}

fn unresolved(name: &str) -> MacroExpansionFact {
    let mut observed = fact(name);
    observed.quality = AnalysisQuality::Unresolved;
    MacroExpansionFact::unresolved(observed)
}

fn repository(name: &str, derivation: MacroDerivation) -> MacroCandidate {
    MacroCandidate {
        observation: fact(name),
        origins: vec![MacroOrigin::Repository {
            package: "fixture".into(),
            directory: ".".into(),
        }],
        derivation,
        written_alias: false,
        definition: None,
        definition_name: None,
        definition_sha256: None,
    }
}

fn compiler(name: &str) -> MacroExpansionFact {
    MacroExpansionFact::with_candidates(
        fact(name),
        vec![MacroCandidate {
            observation: fact(name),
            origins: vec![MacroOrigin::CompilerBuiltin],
            derivation: MacroDerivation::Written,
            written_alias: false,
            definition: None,
            definition_name: None,
            definition_sha256: None,
        }],
    )
}
