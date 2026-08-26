//! Typed place fixtures cover nested fields, parameters, and lexical locals.

use zrail_core::AnalysisQuality;

use crate::source::SourceOperationKind;

use super::{matching, operations};

#[test]
fn nested_and_typed_places_resolve_exact_receiver_types() {
    let facts = operations(
        r"
struct Persistent { current_term: u64 }
struct State { persistent: Persistent }
impl State {
    fn advance(&mut self, state: &mut State) {
        self.persistent.current_term += 1;
        state.persistent.current_term = 2;
        let local: &mut Persistent = &mut self.persistent;
        local.current_term = 3;
    }
}
",
    );

    let writes = matching(
        &facts,
        SourceOperationKind::FieldWrite,
        "Persistent::current_term",
    );
    assert_eq!(writes.len(), 3, "typed receiver identity: {facts:?}");
    assert!(
        writes
            .iter()
            .all(|fact| { fact.identity.quality == AnalysisQuality::Exact && fact.file_local })
    );
    assert!(writes.iter().any(|fact| {
        fact.place.as_ref().is_some_and(|place| {
            place.base_name == "State"
                && place.fields == ["persistent", "current_term"]
                && place.base_span.is_some()
        })
    }));
    assert!(writes.iter().any(|fact| {
        fact.place.as_ref().is_some_and(|place| {
            place.base_name == "Persistent"
                && place.fields == ["current_term"]
                && place.base_span.is_some()
        })
    }));
}

#[test]
fn field_receiver_calls_retain_exact_place_and_written_method() {
    let facts = operations(
        r"
struct Persistent { values: Vec<u64> }
struct State { persistent: Persistent }
impl State {
    fn mutate(&mut self, typed: &mut Persistent) {
        self.persistent.values.push(1);
        typed.values.clear();
    }
}
",
    );

    let calls = matching(
        &facts,
        SourceOperationKind::FieldReceiverCall,
        "Persistent::values",
    );
    assert_eq!(calls.len(), 2, "field receiver identity: {facts:?}");
    assert!(
        calls
            .iter()
            .all(|fact| { fact.identity.quality == AnalysisQuality::Exact && fact.file_local })
    );
    assert_eq!(
        calls
            .iter()
            .filter_map(|fact| fact.method.as_deref())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from(["clear", "push"])
    );
    assert!(calls.iter().all(|fact| fact.place.is_some()));
}

#[test]
fn dereference_stops_field_authority_but_indexing_preserves_it() {
    let facts = operations(
        r"
struct State { ptr: *mut u64, entries: Vec<u64> }
impl State {
    fn mutate(&mut self) {
        *self.ptr = 1;
        let _pointee = &mut *self.ptr;
        self.entries[0] = 2;
        let _entry = &mut self.entries[0];
    }
}
",
    );

    assert!(
        matching(&facts, SourceOperationKind::FieldWrite, "State::ptr").is_empty(),
        "a pointee write does not mutate the pointer field: {facts:?}"
    );
    assert!(
        matching(
            &facts,
            SourceOperationKind::FieldMutableBorrow,
            "State::ptr"
        )
        .is_empty(),
        "a pointee borrow does not mutably borrow the pointer field: {facts:?}"
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldRead, "State::ptr").len(),
        2,
        "dereferencing reads the pointer field"
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "State::entries").len(),
        1,
        "index assignment mutates the aggregate field"
    );
    assert_eq!(
        matching(
            &facts,
            SourceOperationKind::FieldMutableBorrow,
            "State::entries"
        )
        .len(),
        1,
        "indexed mutable borrow retains aggregate authority"
    );
}
