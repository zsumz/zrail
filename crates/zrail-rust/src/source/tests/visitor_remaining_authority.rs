//! Remaining assignee and binding-mode forms must preserve fail-closed field authority.

use zrail_core::AnalysisQuality;

use crate::source::SourceOperationKind;

use super::{matching, operations};

#[test]
fn let_chain_and_match_guard_bindings_shadow_outer_exact_types() {
    let facts = operations(
        r"struct A { secret: usize }
struct B { secret: usize }
fn maybe_b() -> Option<B> { None }
fn transform(_: &B) -> Option<B> { None }
fn inspect(value: A) {
    if let Some(value) = maybe_b()
        && value.secret == 1
    {
        let _ = value.secret;
    } else {
        let _ = value.secret;
    }
    match Some(B { secret: 0 }) {
        candidate if let Some(value) = transform(&candidate)
            && value.secret == 2
            => { let _ = value.secret; }
        _ => { let _ = value.secret; }
    }
}
",
    );

    for line in [7, 9, 15, 16] {
        assert_unresolved_without_exact(&facts, line, "A::secret");
    }
    for line in [11, 17] {
        assert_exact_read(&facts, line, "A::secret");
    }
}

#[test]
fn structural_patterns_distinguish_mutable_shared_value_and_unknown_modes() {
    let facts = operations(
        r"struct Inner { value: usize }
struct State { epoch: usize, inner: Inner }
fn explicit(state: State) {
    let State { epoch: ref mut epoch, inner: Inner { value: ref mut value } } = state;
}
fn implicit(state: &mut State) {
    let State { epoch, inner: Inner { value } } = state;
}
fn shared(state: &State) { let State { epoch, .. } = state; }
fn owned(state: State) { let State { epoch, .. } = state; }
fn unknown() { let _ = |state| { let State { epoch, .. } = state; }; }
",
    );

    assert_exact(
        &facts,
        4,
        SourceOperationKind::FieldMutableBorrow,
        "State::epoch",
    );
    assert_exact(
        &facts,
        4,
        SourceOperationKind::FieldMutableBorrow,
        "Inner::value",
    );
    assert_exact(
        &facts,
        7,
        SourceOperationKind::FieldMutableBorrow,
        "State::epoch",
    );
    assert_exact(
        &facts,
        7,
        SourceOperationKind::FieldMutableBorrow,
        "Inner::value",
    );
    assert_exact_read(&facts, 9, "State::epoch");
    assert_exact_read(&facts, 10, "State::epoch");
    assert_exact_read(&facts, 11, "State::epoch");
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldMutableBorrow
            && fact.identity.span.is_some_and(|span| span.line == 11)
            && fact.identity.name == "State::epoch"
            && fact.identity.quality == AnalysisQuality::Unresolved
            && fact.place.is_none()
    }));
}

#[test]
fn nested_struct_patterns_propagate_field_reference_modes() {
    let facts = operations(
        r"struct Inner { value: usize }
struct Holder<'a> { direct: &'a mut Inner, opaque: Alias<'a> }
type Alias<'a> = &'a mut Inner;
fn inspect(holder: Holder<'_>) {
    let Holder {
        direct: Inner { value: direct },
        opaque: Inner { value: opaque },
    } = holder;
}
",
    );

    assert_exact(
        &facts,
        6,
        SourceOperationKind::FieldMutableBorrow,
        "Holder::direct",
    );
    assert_exact(
        &facts,
        6,
        SourceOperationKind::FieldMutableBorrow,
        "Inner::value",
    );
    assert_exact_read(&facts, 7, "Holder::opaque");
    for name in ["Holder::opaque", "Inner::value"] {
        assert!(facts.iter().any(|fact| {
            fact.kind == SourceOperationKind::FieldMutableBorrow
                && fact.identity.span.is_some_and(|span| span.line == 7)
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Unresolved
                && fact.place.is_none()
        }));
    }
}

#[test]
fn mutable_pattern_authority_covers_every_binding_context_and_nesting() {
    let facts = operations(
        r"struct State { epoch: usize, pair: (usize, usize), values: [usize; 1] }
fn parameter(State { epoch, .. }: &mut State) {}
fn contexts(state: &mut State, owned: State, states: Vec<State>) {
    match state { State { epoch, .. } => { let _ = epoch; } }
    if let State { epoch, .. } = state { let _ = epoch; }
    while let State { epoch, .. } = state { let _ = epoch; break; }
    for State { epoch: ref mut slot, .. } in states { let _ = slot; }
    let State { pair: (ref mut left, _), values: [ref mut first], .. } = owned;
    let _ = |State { epoch, .. }: &mut State| epoch;
}
",
    );

    for line in [2, 4, 5, 6, 7, 9] {
        assert_exact(
            &facts,
            line,
            SourceOperationKind::FieldMutableBorrow,
            "State::epoch",
        );
    }
    for name in ["State::pair", "State::values"] {
        assert_exact(&facts, 8, SourceOperationKind::FieldMutableBorrow, name);
    }
}

#[test]
fn destructuring_assignment_records_only_written_places() {
    let facts = operations(
        r"struct Pair { left: usize, right: usize }
struct TuplePair(usize, usize);
struct Inner { value: usize }
struct Outer { inner: Inner }
struct Slot { value: usize }
struct State { epoch: usize, other: usize, slots: [Slot; 2] }
fn assign(state: &mut State, pair: Pair, rest_pair: Pair, tuple: TuplePair, outer: Outer, array: [usize; 2]) {
    Pair { left: state.epoch, right: _ } = pair;
    Pair { left: state.epoch, .. } = rest_pair;
    TuplePair(state.epoch, state.other) = tuple;
    [state.epoch, state.other] = array;
    (state.epoch, state.other) = (1, 2);
    Outer { inner: Inner { value: state.epoch } } = outer;
    [state.slots[0].value, state.slots[1].value] = array;
}
",
    );

    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "State::epoch").len(),
        6,
        "epoch writes: {facts:#?}"
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "State::other").len(),
        3,
        "other writes: {facts:#?}"
    );
    assert_eq!(
        matching(
            &facts,
            SourceOperationKind::FieldProjectionWrite,
            "State::slots"
        )
        .len(),
        2,
        "indexed backing writes: {facts:#?}"
    );
    assert!(
        matching(&facts, SourceOperationKind::FieldRead, "State::epoch").is_empty(),
        "assignee fields became reads: {facts:#?}"
    );
    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && matches!(
                fact.identity.name.as_str(),
                "Pair" | "TuplePair" | "Outer" | "Inner"
            )
    }));
}

#[test]
fn raw_mutable_address_uses_mutable_borrow_authority() {
    let facts = operations(
        r"struct State { epoch: usize }
fn pointers(state: &mut State) {
    let _mutable = &raw mut state.epoch;
    let _shared = &raw const state.epoch;
}
",
    );

    assert_exact(
        &facts,
        3,
        SourceOperationKind::FieldMutableBorrow,
        "State::epoch",
    );
    assert_exact_read(&facts, 4, "State::epoch");
    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldRead
            && fact.identity.span.is_some_and(|span| span.line == 3)
    }));
}

fn assert_unresolved_without_exact(
    facts: &[crate::source::SourceOperationFact],
    line: usize,
    excluded: &str,
) {
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldRead
            && fact.identity.span.is_some_and(|span| span.line == line)
            && fact.identity.quality == AnalysisQuality::Unresolved
    }));
    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldRead
            && fact.identity.span.is_some_and(|span| span.line == line)
            && fact.identity.name == excluded
            && fact.identity.quality == AnalysisQuality::Exact
    }));
}

fn assert_exact_read(facts: &[crate::source::SourceOperationFact], line: usize, name: &str) {
    assert_exact(facts, line, SourceOperationKind::FieldRead, name);
}

fn assert_exact(
    facts: &[crate::source::SourceOperationFact],
    line: usize,
    kind: SourceOperationKind,
    name: &str,
) {
    assert!(
        facts.iter().any(|fact| {
            fact.kind == kind
                && fact.identity.span.is_some_and(|span| span.line == line)
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Exact
        }),
        "missing {kind:?} {name} on line {line}: {facts:#?}"
    );
}
