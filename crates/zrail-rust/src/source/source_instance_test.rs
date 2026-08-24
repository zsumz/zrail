//! Source-context limits distinguish ordinary input from multiplicative expansion.

use super::*;
use crate::source::{CompilationMode, IncludeOccurrenceId};
use zrail_core::SourceSpan;

#[test]
fn ordinary_base_contexts_are_not_capped_at_four_thousand() {
    let roots = (0..6_001)
        .map(|index| CompilationRoot {
            file: format!("src/unit_{index}.rs"),
            domain: domain(),
        })
        .collect::<Vec<_>>();

    let instances = SourceInstances::build(&roots, &[], &[]);

    assert!(instances.issues().is_empty());
    assert_eq!(instances.metrics().base_contexts, 6_001);
    assert_eq!(instances.metrics().derived_contexts, 0);
}

#[test]
fn include_cycles_retain_the_exact_chain() {
    let includes = vec![
        include("src/lib.rs", "src/part.rs"),
        include("src/part.rs", "src/lib.rs"),
    ];
    let instances = SourceInstances::build(
        &[CompilationRoot {
            file: "src/lib.rs".into(),
            domain: domain(),
        }],
        &[],
        &includes,
    );

    assert_eq!(
        instances.issues(),
        &[SourceInstanceIssue::Cycle {
            chain: vec![
                "src/lib.rs".into(),
                "src/part.rs".into(),
                "src/lib.rs".into()
            ],
        }]
    );
}

#[test]
fn reviewed_derived_context_limit_is_applied_exactly() {
    let roots = ["src/lib.rs", "src/bin_a.rs", "src/bin_b.rs"].map(|file| CompilationRoot {
        file: file.into(),
        domain: domain(),
    });
    let includes = [
        include("src/lib.rs", "src/shared.rs"),
        include("src/bin_a.rs", "src/shared.rs"),
        include("src/bin_b.rs", "src/shared.rs"),
    ];

    let instances = SourceInstances::build_with_limit(&roots, &[], &includes, Some(1));

    assert!(matches!(
        instances.issues(),
        [SourceInstanceIssue::DerivedContextLimit { limit: 1, .. }]
    ));
}

fn include(parent: &str, child: &str) -> CompilationIncludeEdge {
    CompilationIncludeEdge {
        parent: parent.into(),
        child: child.into(),
        domain: domain(),
        guard: SyntaxGuard::Ordinary,
        context: IncludeContext::Items,
        parent_scope: Vec::new(),
        generic_types: Vec::new(),
        include_span: span(),
        occurrence: IncludeOccurrenceId::new(span()),
    }
}

fn domain() -> CompilationDomain {
    CompilationDomain {
        package: "fixture".into(),
        edition: "2024".into(),
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
