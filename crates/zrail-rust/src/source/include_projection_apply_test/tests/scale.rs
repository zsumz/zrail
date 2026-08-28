//! Default work capacity covers repository-scale ordinary resolution.

use super::*;

const ROOTS: usize = 1_000;
const CALLS: usize = 80;

#[test]
fn repeated_resolution_reuses_actual_transition_work() {
    let mut single = fixture_index();
    let single_bindings = bindings(&single);
    assert!(
        single_bindings
            .apply_with_limits(
                &mut single,
                ProjectionLimits {
                    work: 1_000,
                    projected_facts: 10,
                },
            )
            .is_empty()
    );

    let mut repeated = fixture_index();
    repeated.files[0].calls = vec![repeated.files[0].calls[0].clone(); 50];
    let repeated_bindings = bindings(&repeated);
    assert!(
        repeated_bindings
            .apply_with_limits(
                &mut repeated,
                ProjectionLimits {
                    work: 1_000,
                    projected_facts: 10,
                },
            )
            .is_empty()
    );

    assert_eq!(
        repeated.analysis_metrics.projection_work,
        single.analysis_metrics.projection_work
    );
}

#[test]
fn default_work_budget_covers_many_ordinary_compilation_occurrences() {
    let mut limited = scaled_index();
    let limited_bindings = scaled_bindings(&limited);
    let findings = limited_bindings.apply_with_limits(
        &mut limited,
        ProjectionLimits {
            work: 1_000,
            projected_facts: 1,
        },
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("work safety budget"));

    let mut index = scaled_index();
    let bindings = scaled_bindings(&index);
    let findings = bindings.apply(&mut index);
    assert!(findings.is_empty());
    assert_eq!(projected_call_count(&index), 1);
    assert!(index.analysis_metrics.projection_work < 1_000_000);
}

#[test]
fn repositories_without_include_edges_perform_zero_projection_work() {
    let mut index = scaled_index();
    let roots = (0..6_001)
        .map(|unit| CompilationRoot {
            file: format!("src/unit_{unit}.rs"),
            domain: domain(),
        })
        .collect::<Vec<_>>();
    let bindings = IncludeBindings::collect(
        &index,
        &roots,
        &[],
        &[],
        &crate::source::BindingMacroPolicy::default(),
    );

    let findings = bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 0,
            projected_facts: 0,
        },
    );

    assert!(findings.is_empty());
    assert_eq!(index.analysis_metrics.projection_files, 0);
    assert_eq!(index.analysis_metrics.projection_work, 0);
    assert_eq!(index.analysis_metrics.projected_facts, 0);
}

fn scaled_index() -> SourceIndex {
    let mut index = fixture_index();
    let call = index.files[0].calls[0].clone();
    index.files[0].calls = vec![call; CALLS];
    index
}

fn scaled_bindings(index: &SourceIndex) -> IncludeBindings {
    let domain = domain();
    let root = CompilationRoot {
        file: "src/lib.rs".into(),
        domain: domain.clone(),
    };
    let roots = vec![root; ROOTS];
    let includes = [CompilationIncludeEdge {
        parent: "src/lib.rs".into(),
        child: "src/imports.rs".into(),
        domain,
        guard: SyntaxGuard::Ordinary,
        context: IncludeContext::Items,
        parent_scope: Vec::new(),
        generic_types: Vec::new(),
        prelude_value_shadows: Vec::new(),
        include_span: span(),
        occurrence: IncludeOccurrenceId::new(span()),
    }];
    IncludeBindings::collect(
        index,
        &roots,
        &[],
        &includes,
        &crate::source::BindingMacroPolicy::default(),
    )
}
