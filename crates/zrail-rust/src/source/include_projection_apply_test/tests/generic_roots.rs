//! Generic call roots replace stale physical spellings across expression includes.

use super::*;

#[test]
fn expression_include_replaces_generic_associated_spelling_with_synthetic_identity() {
    let mut index = SourceIndex {
        files: vec![
            file("src/lib.rs", Vec::new(), Vec::new()),
            file(
                "src/expr.rs",
                vec![ObservedFact {
                    name: "Choice::ready".into(),
                    written: Some("Choice::ready".into()),
                    implicit_prelude: crate::source::ImplicitPreludeEligibility::LocalShadow,
                    canonical: Vec::new(),
                    span: Some(span()),
                    quality: AnalysisQuality::Exact,
                    guard: SyntaxGuard::Ordinary,
                    lexical_scope: Vec::new(),
                    namespace: crate::source::FactNamespace::Unknown,
                    generic_shadow: None,
                    associated_candidates: Vec::new(),
                    inherits_parent_context: true,
                }],
                Vec::new(),
            ),
        ],
        findings: Vec::new(),
        analysis_metrics: SourceAnalysisMetrics::default(),
    };
    index.files[1].syntax = SourceSyntax::Expression;
    let domain = domain();
    let bindings = IncludeBindings::collect(
        &index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            syntax: SourceSyntax::Items,
            domain: domain.clone(),
        }],
        &[],
        &[CompilationIncludeEdge {
            parent: "src/lib.rs".into(),
            parent_syntax: SourceSyntax::Items,
            child: "src/expr.rs".into(),
            child_syntax: SourceSyntax::Expression,
            domain,
            guard: SyntaxGuard::Ordinary,
            context: IncludeContext::Expression,
            parent_scope: Vec::new(),
            generic_types: vec!["Choice".into()],
            generic_values: Vec::new(),
            trait_bounds: Vec::new(),
            current_self: None,
            inherits_parent_context: true,
            value_shadows: Vec::new(),
            include_span: span(),
            occurrence: IncludeOccurrenceId::new(span()),
        }],
        &crate::source::BindingMacroPolicy::default(),
    );

    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 1_000,
            projected_facts: 10,
        },
    );

    assert!(findings.is_empty(), "{findings:#?}");
    assert_eq!(index.files[1].calls.len(), 1, "{:#?}", index.files[1].calls);
    assert_eq!(
        index.files[1].calls[0].name,
        "<type-parameter Choice>::ready"
    );
    assert_eq!(
        index.files[1].calls[0].generic_shadow,
        Some(crate::source::GenericRootShadow::TypeParameter)
    );
}
