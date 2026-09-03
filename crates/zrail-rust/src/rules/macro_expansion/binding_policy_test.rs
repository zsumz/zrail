//! Namespace opacity clears only for exact, canonically covered expansion authority.

use std::collections::BTreeMap;

use zrail_core::{
    AnalysisContract, AnalysisQuality, Contract, CrateRootSource, CycleMode, DependenciesContract,
    DependencyMode, ExactMode, FacadeMode, HygieneContract, LintSuppressionMode, MacroBindingMode,
    MacroExpansionAllow, MacroExpansionBindings, MacroExpansionContract, MacroExpansionMode,
    MacroInputMode, ModuleDocsMode, PolicyMode, RepositoryContract, RustSourceContract,
    SourceContract, SymlinkMode, TestMode,
};

use crate::{
    cargo::DependencySource,
    inventory::FileClass,
    source::{
        FactNamespace, MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin,
        ObservedFact, Reachability, ReachabilityKind, RustFileFacts, SourceAnalysisMetrics,
        SourceIndex, SourceSyntax, SyntaxGuard,
    },
};

use super::build;

#[test]
fn conservative_review_confidence_never_clears_opacity() {
    let conservative = expansion(
        "trusted::derive",
        "trusted::derive",
        AnalysisQuality::Conservative,
        false,
    );
    let policy = build(
        &contract(vec![clean_allowance("trusted::derive")]),
        &source(conservative.clone()),
        None,
    );

    assert!(policy.retains_opacity("src/lib.rs", &conservative.observation));

    let exact = expansion(
        "trusted::derive",
        "trusted::derive",
        AnalysisQuality::Exact,
        false,
    );
    let exact_policy = build(
        &contract(vec![clean_allowance("trusted::derive")]),
        &source(exact.clone()),
        None,
    );
    assert!(!exact_policy.retains_opacity("src/lib.rs", &exact.observation));
}

#[test]
fn written_alias_cannot_cover_an_unmatched_canonical_candidate() {
    let aliased = expansion("reviewed", "trusted::derive", AnalysisQuality::Exact, true);
    let alias_policy = build(
        &contract(vec![clean_allowance("reviewed")]),
        &source(aliased.clone()),
        None,
    );

    assert!(alias_policy.retains_opacity("src/lib.rs", &aliased.observation));

    let canonical_policy = build(
        &contract(vec![clean_allowance("trusted::derive")]),
        &source(aliased.clone()),
        None,
    );
    assert!(!canonical_policy.retains_opacity("src/lib.rs", &aliased.observation));
}

#[test]
fn compiler_builtins_preserve_bindings_without_contract_allowances() {
    let builtin = MacroExpansionFact::compiler_builtin(fact("Debug", AnalysisQuality::Exact));
    let policy = build(&contract(Vec::new()), &source(builtin.clone()), None);

    assert!(!policy.retains_opacity("src/lib.rs", &builtin.observation));
}

#[test]
fn source_operation_closure_requires_an_exact_bound_attestation() {
    let exact = expansion(
        "trusted::derive",
        "trusted::derive",
        AnalysisQuality::Exact,
        false,
    );
    let index = source(exact.clone());
    let mut attested = clean_allowance("trusted::derive");
    attested.bindings = MacroExpansionBindings::Opaque;
    attested.source_operations = zrail_core::MacroSourceOperations::None;
    attested.binding = MacroBindingMode::Conservative;

    assert!(crate::rules::closes_source_operations(
        &contract(vec![attested.clone()]),
        &index,
        None,
        &exact,
    ));
    let uncertain = expansion(
        "trusted::derive",
        "trusted::derive",
        AnalysisQuality::Conservative,
        false,
    );
    assert!(!crate::rules::closes_source_operations(
        &contract(vec![attested]),
        &source(uncertain.clone()),
        None,
        &uncertain,
    ));
    assert!(!crate::rules::closes_source_operations(
        &contract(vec![clean_allowance("trusted::derive")]),
        &index,
        None,
        &exact,
    ));
}

pub(in crate::rules::macro_expansion) fn expansion(
    written: &str,
    canonical: &str,
    quality: AnalysisQuality,
    written_alias: bool,
) -> MacroExpansionFact {
    MacroExpansionFact::with_candidates(
        fact(written, quality),
        vec![MacroCandidate {
            observation: fact(canonical, quality),
            origins: vec![MacroOrigin::External {
                package: "trusted".into(),
                source: registry_source(),
            }],
            derivation: if written_alias {
                MacroDerivation::ExactImport
            } else {
                MacroDerivation::DependencyRoot
            },
            written_alias,
            definition: None,
            definition_name: None,
            definition_sha256: None,
        }],
    )
}

fn fact(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        implicit_prelude: crate::source::ImplicitPreludeEligibility::Disabled,
        canonical: Vec::new(),
        span: None,
        quality,
        guard: SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Unknown,
        generic_shadow: None,
        associated_candidates: Vec::new(),
        inherits_parent_context: true,
    }
}

pub(in crate::rules::macro_expansion) fn source(expansion: MacroExpansionFact) -> SourceIndex {
    SourceIndex {
        files: vec![RustFileFacts {
            relative: "src/lib.rs".into(),
            packages: Vec::new(),
            class: FileClass::Facade,
            reachability: Reachability::from_kind(ReachabilityKind::Production),
            syntax: SourceSyntax::Items,
            lines: 1,
            module_docs: true,
            paths: Vec::new(),
            calls: Vec::new(),
            call_resolutions: Vec::new(),
            methods: Vec::new(),
            operations: Vec::new(),
            macros: Vec::new(),
            macro_imports: Vec::new(),
            macro_expansions: vec![expansion],
            opaque_macro_inputs: Vec::new(),
            macro_definitions: Vec::new(),
            import_bindings: Vec::new(),
            associated_items: Vec::new(),
            trait_declarations: Vec::new(),
            glob_imports: Vec::new(),
            inline_module_scopes: Vec::new(),
            prelude_directives: Vec::new(),
            compile_effects: Vec::new(),
            lint_suppressions: Vec::new(),
            unsafe_constructs: Vec::new(),
            async_syntax: Vec::new(),
            type_policy: crate::source::TypePolicyFacts::default(),
            tests: Vec::new(),
            modules: Vec::new(),
            includes: Vec::new(),
            item_macros: Vec::new(),
            opaque_binding_macros: Vec::new(),
            facade_implementation: Vec::new(),
        }],
        findings: Vec::new(),
        analysis_metrics: SourceAnalysisMetrics::default(),
    }
}

pub(in crate::rules::macro_expansion) fn clean_allowance(name: &str) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: name.into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::None,
        async_syntax: zrail_core::MacroAsyncSyntax::Opaque,
        duplication_effect: zrail_core::MacroDuplicationEffect::Opaque,
        source_operations: zrail_core::MacroSourceOperations::Opaque,
        field_mutation: zrail_core::MacroFieldMutation::Opaque,
        definition: None,
        source: Some(CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: "=1.0.0".into(),
        }),
        reason: "Reviewed expansion preserves the ordinary namespace exactly.".into(),
    }
}

fn registry_source() -> DependencySource {
    DependencySource::Registry {
        registry: None,
        index: None,
        requirement: "=1.0.0".into(),
    }
}

pub(in crate::rules::macro_expansion) fn contract(allow: Vec<MacroExpansionAllow>) -> Contract {
    Contract {
        schema: 1,
        adapters: vec!["rust".into()],
        repository: RepositoryContract {
            roots: vec![".".into()],
            exclude: Vec::new(),
            workspace_members: ExactMode::Exact,
            nested_git: PolicyMode::Deny,
            submodules: PolicyMode::Deny,
            symlinks: SymlinkMode::Inside,
        },
        dependencies: DependenciesContract {
            mode: DependencyMode::Observed,
            unassigned_packages: PolicyMode::Allow,
            cycles: CycleMode::Deny,
            crate_roots: Vec::new(),
        },
        analysis: AnalysisContract::default(),
        source: SourceContract {
            rust: RustSourceContract {
                module_docs: ModuleDocsMode::Allow,
                facades: FacadeMode::Allow,
                entrypoints: FacadeMode::Allow,
                tests: TestMode::Allow,
                file_roles: Vec::new(),
                generated: Vec::new(),
                out_dir: Vec::new(),
                item_macros: Vec::new(),
                test_mirrors: Vec::new(),
                feature_worlds: Vec::new(),
                macros: MacroExpansionContract {
                    mode: MacroExpansionMode::DenyUnreviewed,
                    allow,
                },
                duplication: zrail_core::RustDuplicationContract::default(),
                types: Vec::new(),
                hygiene: HygieneContract {
                    unsafe_code: PolicyMode::Deny,
                    lint_suppressions: LintSuppressionMode::Allow,
                    deny_methods: Vec::new(),
                    deny_macros: Vec::new(),
                    glob_imports: zrail_core::GlobImportMode::Allow,
                },
                size: None,
            },
        },
        profiles: BTreeMap::new(),
        layers: Vec::new(),
        dependency_rules: Vec::new(),
        scopes: Vec::new(),
        owners: Vec::new(),
        ratchets: Vec::new(),
        gates: Vec::new(),
        invariants: Vec::new(),
    }
}
