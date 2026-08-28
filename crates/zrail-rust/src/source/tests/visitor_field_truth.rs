//! Adversarial field syntax must be present and honestly qualified.

use zrail_core::AnalysisQuality;

use crate::source::SourceOperationKind;

use super::operations;

#[test]
fn unsupported_bases_retain_every_named_field_operation() {
    let facts = operations(
        r"struct State { field: usize, inner: Inner }
struct Inner { field: usize }
fn inspect(vec: Vec<State>, map: Map, key: Key) {
    let _ = vec[0].field;
    vec[0].field = 1;
    let _ = &mut vec[0].field;
    vec[0].field.clear();
    let _ = map[key].inner.field;
    let _ = factory().field;
    get_mut().field.clear();
    let _ = (array())[0].field;
}
",
    );

    for (line, kind) in [
        (4, SourceOperationKind::FieldRead),
        (5, SourceOperationKind::FieldWrite),
        (6, SourceOperationKind::FieldMutableBorrow),
        (7, SourceOperationKind::FieldReceiverCall),
        (8, SourceOperationKind::FieldRead),
        (9, SourceOperationKind::FieldRead),
        (10, SourceOperationKind::FieldReceiverCall),
        (11, SourceOperationKind::FieldRead),
    ] {
        assert!(
            facts.iter().any(|fact| {
                fact.kind == kind
                    && fact.identity.written.as_deref() == Some("field")
                    && fact.identity.span.is_some_and(|span| span.line == line)
                    && fact.identity.quality == AnalysisQuality::Unresolved
                    && fact.place.is_none()
            }),
            "missing unresolved {kind:?} on line {line}: {facts:#?}"
        );
    }
}

#[test]
fn exact_field_identity_requires_a_local_declaration_with_that_member() {
    let facts = operations(
        r"use std::{pin::Pin, sync::Arc};
struct State { epoch: usize }
struct StatePtr(State);
struct Wrapper { epoch: usize, inner: State }
struct Nested { inner: StatePtr }
impl core::ops::Deref for StatePtr {
    type Target = State;
    fn deref(&self) -> &State { &self.0 }
}
fn inspect(state: State, ptr: StatePtr, boxed: Box<State>, arc: Arc<State>, pin: Pin<&State>, external: External, wrapper: Wrapper, nested: Nested) {
    let _ = state.epoch;
    let _ = ptr.epoch;
    let _ = boxed.epoch;
    let _ = arc.epoch;
    let _ = pin.epoch;
    let _ = external.epoch;
    let _ = wrapper.epoch;
    let _ = wrapper.inner.epoch;
    let _ = nested.inner.epoch;
}
",
    );

    assert_exact(&facts, 11, "State::epoch");
    assert_unresolved(&facts, 12, "StatePtr::epoch");
    for line in 13..=16 {
        assert!(
            facts.iter().any(|fact| {
                fact.kind == SourceOperationKind::FieldRead
                    && fact.identity.span.is_some_and(|span| span.line == line)
                    && fact.identity.written.as_deref() == Some("epoch")
                    && fact.identity.quality == AnalysisQuality::Unresolved
            }),
            "autoderef boundary on line {line}: {facts:#?}"
        );
    }
    assert_exact(&facts, 17, "Wrapper::epoch");
    assert_exact(&facts, 18, "State::epoch");
    assert_unresolved(&facts, 19, "StatePtr::epoch");
}

#[test]
fn typed_generic_receiver_keeps_synthetic_identity_before_local_catalog_lookup() {
    let facts = operations(
        r"use core::ops::Deref;
struct State { secret: u64 }
struct Actual { secret: u64 }
fn inspect<State>(state: State) -> u64
where State: Deref<Target = Actual>
{
    state.secret
}
",
    );

    let field = facts
        .iter()
        .find(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.written.as_deref() == Some("secret")
        })
        .expect("generic field read");
    assert_eq!(field.identity.name, "<type-parameter State>::secret");
    assert_eq!(field.identity.quality, AnalysisQuality::Unresolved);
    assert!(!field.file_local);
}

#[test]
fn unknown_shadows_tombstone_outer_types_and_cfg_bindings_keep_both_worlds() {
    let facts = operations(
        r#"struct A { secret: usize }
struct B { secret: usize }
fn inspect(original: A, values: Vec<B>) {
    let value: A = original;
    {
        let value = make_b();
        let _ = value.secret;
    }
    values.iter().for_each(|value| { let _ = value.secret; });
    let _ = value.secret;
    #[cfg(feature = "a")]
    let state: A = make_a();
    #[cfg(not(feature = "a"))]
    let state: B = make_b();
    let _ = state.secret;
}
"#,
    );

    for line in [7, 9] {
        assert!(
            facts.iter().any(|fact| {
                fact.kind == SourceOperationKind::FieldRead
                    && fact.identity.span.is_some_and(|span| span.line == line)
                    && fact.identity.quality == AnalysisQuality::Unresolved
                    && fact.identity.name == "<unresolved>::secret"
            }),
            "shadow on line {line}: {facts:#?}"
        );
    }
    assert_exact(&facts, 10, "A::secret");
    let world_facts = facts
        .iter()
        .filter(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.span.is_some_and(|span| span.line == 15)
        })
        .collect::<Vec<_>>();
    assert_eq!(world_facts.len(), 2, "feature candidates: {facts:#?}");
    assert!(
        world_facts
            .iter()
            .any(|fact| fact.identity.name == "A::secret")
    );
    assert!(
        world_facts
            .iter()
            .any(|fact| fact.identity.name == "B::secret")
    );
    assert!(
        world_facts
            .iter()
            .all(|fact| fact.identity.quality == AnalysisQuality::Exact)
    );
    assert_ne!(world_facts[0].identity.guard, world_facts[1].identity.guard);
}

#[test]
fn structural_patterns_emit_field_reads_and_shadow_outer_bindings() {
    let facts = operations(
        r"struct State { epoch: usize }
struct Other { secret: usize }
fn parameter(State { epoch: renamed, .. }: State) { let _ = renamed; }
fn inspect(state: State, states: Vec<State>, outer: Other) {
    let epoch: Other = outer;
    let State { epoch: renamed, .. } = state;
    match state { State { epoch, .. } => { let _ = epoch.secret; } }
    if let State { epoch: 7, .. } = state {}
    while let State { epoch: 7, .. } = state { break; }
    for State { epoch: ref mut item, .. } in states { let _ = item; }
    let State { epoch: final_epoch, .. } = state else { return; };
    let _ = |State { epoch, .. }: State| epoch;
    let _ = matches!(state, State { epoch: 7, .. });
}
",
    );

    for line in [3, 6, 7, 8, 9, 11, 12, 13] {
        assert_exact(&facts, line, "State::epoch");
    }
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldMutableBorrow
            && fact.identity.span.is_some_and(|span| span.line == 10)
            && fact.identity.name == "State::epoch"
            && fact.identity.quality == AnalysisQuality::Exact
    }));
    assert!(
        facts.iter().any(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.span.is_some_and(|span| span.line == 7)
                && fact.identity.name == "<unresolved>::secret"
                && fact.identity.quality == AnalysisQuality::Unresolved
        }),
        "match binding fell through to outer epoch: {facts:#?}"
    );
}

fn assert_exact(facts: &[crate::source::SourceOperationFact], line: usize, name: &str) {
    assert!(
        facts.iter().any(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.span.is_some_and(|span| span.line == line)
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Exact
        }),
        "missing exact {name} on line {line}: {facts:#?}"
    );
}

fn assert_unresolved(facts: &[crate::source::SourceOperationFact], line: usize, name: &str) {
    assert!(
        facts.iter().any(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.span.is_some_and(|span| span.line == line)
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Unresolved
        }),
        "missing unresolved {name} on line {line}: {facts:#?}"
    );
}
