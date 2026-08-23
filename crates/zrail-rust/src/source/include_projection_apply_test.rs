//! Small injected budgets prove repository-wide projection failure semantics.

use zrail_core::{AnalysisQuality, SourceSpan};

use crate::inventory::FileClass;

use super::*;
use crate::source::{
    CompilationDomain, CompilationIncludeEdge, CompilationMode, CompilationRoot, ImportBindingFact,
    IncludeContext, IncludeOccurrenceId, Reachability, SourceSyntax, SyntaxGuard,
    include_bindings::IncludeBindings, include_projection_budget::ProjectionLimits,
};

#[test]
fn work_exhaustion_is_transactional_and_independent_of_file_order() {
    let mut forward = fixture_index();
    let forward_bindings = bindings(&forward);
    let before = fact_lengths(&forward);
    let forward_findings = forward_bindings.apply_with_limits(
        &mut forward,
        ProjectionLimits {
            work: 0,
            total_facts: 100,
        },
    );

    let mut reversed = fixture_index();
    reversed.files.reverse();
    let reversed_bindings = bindings(&reversed);
    let reversed_before = fact_lengths(&reversed);
    let reversed_findings = reversed_bindings.apply_with_limits(
        &mut reversed,
        ProjectionLimits {
            work: 0,
            total_facts: 100,
        },
    );

    assert_eq!(forward_findings.len(), 1);
    assert_eq!(forward_findings, reversed_findings);
    assert!(forward_findings[0].message.contains("work safety budget"));
    assert_eq!(fact_lengths(&forward), before);
    assert_eq!(fact_lengths(&reversed), reversed_before);
}

#[test]
fn fact_exhaustion_retains_no_partial_projection() {
    let mut index = fixture_index();
    let bindings = bindings(&index);
    let before = fact_lengths(&index);
    let physical_facts = index.files.iter().map(fact_count).sum();

    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 1_000,
            total_facts: physical_facts,
        },
    );

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("fact safety budget"));
    assert_eq!(fact_lengths(&index), before);
}

#[test]
fn successful_projection_stays_inside_the_total_fact_limit() {
    let mut index = fixture_index();
    let bindings = bindings(&index);
    let physical_facts = index.files.iter().map(fact_count).sum::<usize>();

    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 1_000,
            total_facts: physical_facts + 1,
        },
    );

    assert!(findings.is_empty());
    assert_eq!(
        index.files.iter().map(fact_count).sum::<usize>(),
        physical_facts + 1
    );
    assert!(index.files.iter().any(|file| {
        file.calls
            .iter()
            .any(|fact| fact.name == "std::process::Command::new")
    }));
}

fn bindings(index: &SourceIndex) -> IncludeBindings {
    let domain = domain();
    IncludeBindings::collect(
        index,
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            domain: domain.clone(),
        }],
        &[],
        &[CompilationIncludeEdge {
            parent: "src/lib.rs".into(),
            child: "src/imports.rs".into(),
            domain,
            context: IncludeContext::Items,
            parent_scope: Vec::new(),
            include_span: span(),
            occurrence: IncludeOccurrenceId::new(span()),
        }],
    )
}

fn fixture_index() -> SourceIndex {
    SourceIndex {
        files: vec![
            file(
                "src/lib.rs",
                vec![ObservedFact {
                    name: "Spawn::new".into(),
                    written: Some("Spawn::new".into()),
                    canonical: Vec::new(),
                    span: Some(span()),
                    quality: AnalysisQuality::Exact,
                    guard: SyntaxGuard::Ordinary,
                    lexical_scope: Vec::new(),
                }],
                Vec::new(),
            ),
            file(
                "src/imports.rs",
                Vec::new(),
                vec![ImportBindingFact {
                    name: Some("Spawn".into()),
                    target: "std::process::Command".into(),
                    quality: AnalysisQuality::Exact,
                    guard: SyntaxGuard::Ordinary,
                    lexical_scope: Vec::new(),
                }],
            ),
        ],
        findings: Vec::new(),
    }
}

fn file(
    relative: &str,
    calls: Vec<ObservedFact>,
    import_bindings: Vec<ImportBindingFact>,
) -> RustFileFacts {
    RustFileFacts {
        relative: relative.into(),
        packages: Vec::new(),
        class: FileClass::Implementation,
        reachability: Reachability::UNREACHABLE,
        syntax: SourceSyntax::Items,
        lines: 1,
        module_docs: true,
        paths: Vec::new(),
        calls,
        methods: Vec::new(),
        macros: Vec::new(),
        macro_imports: Vec::new(),
        macro_expansions: Vec::new(),
        opaque_macro_inputs: Vec::new(),
        macro_definitions: Vec::new(),
        import_bindings,
        inline_module_scopes: Vec::new(),
        compile_effects: Vec::new(),
        lint_suppressions: Vec::new(),
        unsafe_constructs: Vec::new(),
        tests: Vec::new(),
        modules: Vec::new(),
        includes: Vec::new(),
        item_macros: Vec::new(),
        facade_implementation: Vec::new(),
    }
}

fn fact_lengths(index: &SourceIndex) -> Vec<(String, usize, usize)> {
    let mut lengths = index
        .files
        .iter()
        .map(|file| (file.relative.clone(), file.paths.len(), file.calls.len()))
        .collect::<Vec<_>>();
    lengths.sort();
    lengths
}

fn domain() -> CompilationDomain {
    CompilationDomain {
        package: "fixture".into(),
        target: "fixture".into(),
        mode: CompilationMode::Library,
    }
}

const fn span() -> SourceSpan {
    SourceSpan {
        line: 1,
        column: 0,
        end_line: 1,
        end_column: 1,
    }
}
