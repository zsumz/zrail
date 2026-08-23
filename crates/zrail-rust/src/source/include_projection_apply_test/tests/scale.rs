//! Default work capacity covers repository-scale ordinary resolution.

use super::*;

const ROOTS: usize = 1_000;
const CALLS: usize = 80;

#[test]
fn default_work_budget_covers_many_ordinary_compilation_occurrences() {
    let mut limited = scaled_index();
    let limited_bindings = scaled_bindings(&limited);
    let physical_facts = limited.files.iter().map(fact_count).sum::<usize>();
    let findings = limited_bindings.apply_with_limits(
        &mut limited,
        ProjectionLimits {
            work: 1_000_000,
            total_facts: physical_facts + 1,
        },
    );
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("work safety budget"));

    let mut index = scaled_index();
    let bindings = scaled_bindings(&index);
    let findings = bindings.apply(&mut index);
    assert!(findings.is_empty());
    assert_eq!(projected_call_count(&index), 1);
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
