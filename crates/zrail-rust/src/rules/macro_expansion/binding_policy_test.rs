//! Namespace opacity clears only for exact, canonically covered expansion authority.

use std::collections::BTreeMap;

use zrail_core::{
    AnalysisQuality, Contract, CrateRootSource, CycleMode, DependenciesContract, DependencyMode,
    ExactMode, FacadeMode, HygieneContract, LintSuppressionMode, MacroBindingMode,
    MacroExpansionAllow, MacroExpansionBindings, MacroExpansionContract, MacroExpansionMode,
    MacroInputMode, ModuleDocsMode, PolicyMode, RepositoryContract, RustSourceContract,
    SourceContract, SymlinkMode, TestMode,
};

use crate::{
    cargo::DependencySource,
    inventory::FileClass,
    source::{
        FactNamespace, MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin,
        ObservedFact, Reachability, ReachabilityKind, RustFileFacts, SourceIndex, SourceSyntax,
        SyntaxGuard,
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
    );
    assert!(!exact_policy.retains_opacity("src/lib.rs", &exact.observation));
}

#[test]
fn written_alias_cannot_cover_an_unmatched_canonical_candidate() {
    let aliased = expansion("reviewed", "trusted::derive", AnalysisQuality::Exact, true);
    let alias_policy = build(
        &contract(vec![clean_allowance("reviewed")]),
        &source(aliased.clone()),
    );

    assert!(alias_policy.retains_opacity("src/lib.rs", &aliased.observation));

    let canonical_policy = build(
        &contract(vec![clean_allowance("trusted::derive")]),
        &source(aliased.clone()),
    );
    assert!(!canonical_policy.retains_opacity("src/lib.rs", &aliased.observation));
}

fn expansion(
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
        }],
    )
}

fn fact(name: &str, quality: AnalysisQuality) -> ObservedFact {
    ObservedFact {
        name: name.into(),
        written: None,
        canonical: Vec::new(),
        span: None,
        quality,
        guard: SyntaxGuard::Ordinary,
        lexical_scope: Vec::new(),
        namespace: FactNamespace::Unknown,
    }
}

fn source(expansion: MacroExpansionFact) -> SourceIndex {
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
            methods: Vec::new(),
            macros: Vec::new(),
            macro_imports: Vec::new(),
            macro_expansions: vec![expansion],
            opaque_macro_inputs: Vec::new(),
            macro_definitions: Vec::new(),
            import_bindings: Vec::new(),
            inline_module_scopes: Vec::new(),
            compile_effects: Vec::new(),
            lint_suppressions: Vec::new(),
            unsafe_constructs: Vec::new(),
            tests: Vec::new(),
            modules: Vec::new(),
            includes: Vec::new(),
            item_macros: Vec::new(),
            opaque_binding_macros: Vec::new(),
            facade_implementation: Vec::new(),
        }],
        findings: Vec::new(),
    }
}

fn clean_allowance(name: &str) -> MacroExpansionAllow {
    MacroExpansionAllow {
        name: name.into(),
        inputs: MacroInputMode::Inspect,
        binding: MacroBindingMode::Exact,
        bindings: MacroExpansionBindings::None,
        definition: None,
        source: Some(CrateRootSource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        }),
        reason: "Reviewed expansion preserves the ordinary namespace exactly.".into(),
    }
}

fn registry_source() -> DependencySource {
    DependencySource::Registry {
        registry: None,
        index: None,
        requirement: "1".into(),
    }
}

fn contract(allow: Vec<MacroExpansionAllow>) -> Contract {
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
                macros: MacroExpansionContract {
                    mode: MacroExpansionMode::DenyUnreviewed,
                    allow,
                },
                hygiene: HygieneContract {
                    unsafe_code: PolicyMode::Deny,
                    lint_suppressions: LintSuppressionMode::Allow,
                    deny_methods: Vec::new(),
                    deny_macros: Vec::new(),
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
