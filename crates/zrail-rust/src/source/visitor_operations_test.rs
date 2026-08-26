//! Operation extraction separates exact subjects from syntactic method names.

use syn::visit::Visit;
use zrail_core::AnalysisQuality;

use crate::source::{SourceOperationKind, SyntaxGuard, imports::ImportMap};

use super::FactVisitor;

#[test]
fn constructions_cover_struct_tuple_variant_unit_and_self_forms() {
    let facts = operations(
        r"
struct Record { value: usize }
struct Tuple(usize);
enum Choice { Tuple(usize), Record { value: usize }, Unit }
impl Record { fn make() -> Self { Self { value: 1 } } }
impl Tuple { fn make() -> Self { Self(1) } }
fn make() {
    let _ = Record { value: 1 };
    let _ = Tuple(1);
    let _ = Choice::Tuple(1);
    let _ = Choice::Record { value: 1 };
    let _ = Choice::Unit;
}
",
    );
    let constructions = facts
        .iter()
        .filter(|fact| fact.kind == SourceOperationKind::TypeConstruction)
        .collect::<Vec<_>>();

    for name in [
        "Record",
        "Tuple",
        "Choice::Tuple",
        "Choice::Record",
        "Choice::Unit",
    ] {
        assert!(
            constructions.iter().any(|fact| {
                fact.identity.name == name
                    && fact.identity.quality == AnalysisQuality::Exact
                    && fact.file_local
            }),
            "missing {name}: {constructions:?}",
        );
    }
    assert_eq!(
        constructions
            .iter()
            .filter(|fact| fact.identity.name == "Record")
            .count(),
        2,
    );
    assert_eq!(
        constructions
            .iter()
            .filter(|fact| fact.identity.name == "Tuple")
            .count(),
        2,
    );
}

#[test]
fn field_writes_and_mutable_borrows_share_exact_self_identity() {
    let facts = operations(
        r"
struct State { epoch: usize }
impl State {
    fn advance(&mut self) {
        self.epoch = 1;
        self.epoch += 1;
        let _ = &mut self.epoch;
        let _ = std::mem::replace(&mut self.epoch, 2);
        self.commit();
    }
    fn commit(&mut self) {}
}
",
    );
    let writes = matching(&facts, SourceOperationKind::FieldWrite, "State::epoch");
    let borrows = matching(
        &facts,
        SourceOperationKind::FieldMutableBorrow,
        "State::epoch",
    );
    let methods = matching(&facts, SourceOperationKind::MethodCall, "commit");

    assert_eq!(writes.len(), 2);
    assert_eq!(borrows.len(), 2);
    assert_eq!(methods.len(), 1);
    assert!(
        writes
            .iter()
            .chain(&borrows)
            .all(|fact| { fact.identity.quality == AnalysisQuality::Exact && fact.file_local })
    );
    assert_eq!(methods[0].identity.quality, AnalysisQuality::Exact);
    assert!(!methods[0].file_local);
}

#[test]
fn field_reads_exclude_write_and_mutable_borrow_places() {
    let facts = operations(
        r"
struct State { epoch: usize, index: usize, values: [usize; 2] }
impl State {
    fn inspect(&mut self) {
        let _ = self.epoch;
        self.epoch = self.epoch;
        self.epoch += 1;
        let _ = &mut self.epoch;
        let _ = std::mem::replace(&mut self.epoch, 2);
        let _ = &self.epoch;
        self.values[self.index] = 1;
    }
}
",
    );

    assert_eq!(
        matching(&facts, SourceOperationKind::FieldRead, "State::epoch").len(),
        3
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldRead, "State::index").len(),
        1
    );
    assert!(
        matching(&facts, SourceOperationKind::FieldRead, "State::values").is_empty(),
        "assignment place was counted as a read: {facts:?}"
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "State::values").len(),
        1,
        "indexed backing storage had no write authority: {facts:?}"
    );
}

#[test]
fn structured_places_retain_backing_authority_and_value_reads() {
    let facts = operations(
        r"
struct Inner { nested: usize, deref: usize }
struct State {
    values: [usize; 2],
    index: usize,
    outer: Inner,
    pointer: Box<Inner>,
    tuple: (usize,),
}
impl State {
    fn mutate(&mut self, next: usize) {
        self.values[self.index] = next;
        self.values[self.index] += 1;
        self.outer.nested = next;
        (*self.pointer).deref = next;
        self.tuple.0 = next;
        let _ = std::mem::replace(&mut self.values[self.index], next);
    }
}
",
    );

    for (kind, name, count) in [
        (SourceOperationKind::FieldWrite, "State::values", 2),
        (SourceOperationKind::FieldWrite, "State::outer", 1),
        (SourceOperationKind::FieldWrite, "State::pointer", 0),
        (SourceOperationKind::FieldWrite, "State::tuple", 1),
        (SourceOperationKind::FieldMutableBorrow, "State::values", 1),
        (SourceOperationKind::FieldRead, "State::index", 3),
        (SourceOperationKind::FieldRead, "State::pointer", 1),
    ] {
        assert_eq!(
            matching(&facts, kind, name).len(),
            count,
            "{kind:?} {name}: {facts:?}"
        );
    }
    for name in ["State::values", "State::outer", "State::tuple"] {
        assert!(
            matching(&facts, SourceOperationKind::FieldRead, name).is_empty(),
            "place projection was counted as a read for {name}: {facts:?}",
        );
    }
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "Inner::nested").len(),
        1,
        "nested local field type was not retained: {facts:?}",
    );
    assert_eq!(
        matching(&facts, SourceOperationKind::FieldWrite, "Box::deref").len(),
        1,
        "opaque generic dereference did not retain its unresolved candidate: {facts:?}",
    );
}

#[test]
fn imported_structs_resolve_but_unknown_constructor_and_receiver_candidates_do_not() {
    let facts = operations(
        r"
use crate::model::Record as Alias;
fn work(value: Unknown) {
    let _ = Alias { value: 1 };
    let _ = External(1);
    let _ = external::Choice::Unit;
    value.epoch = 1;
}
",
    );

    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.name == "crate::model::Record"
            && fact.identity.quality == AnalysisQuality::Exact
            && !fact.file_local
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.name == "External"
            && fact.identity.quality == AnalysisQuality::Unresolved
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.name == "external::Choice::Unit"
            && fact.identity.quality == AnalysisQuality::Unresolved
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldWrite
            && fact.identity.name == "Unknown::epoch"
            && fact.identity.quality == AnalysisQuality::Unresolved
    }));
}

#[test]
fn operation_facts_preserve_test_only_guards() {
    let facts = operations(
        r"
struct State { epoch: usize }
#[cfg(test)]
fn proof() { let _ = State { epoch: 1 }; }
",
    );

    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.guard == SyntaxGuard::TestOnly
    }));
}

fn operations(source: &str) -> Vec<crate::source::SourceOperationFact> {
    let syntax = syn::parse_file(source).expect("parse operation fixture");
    let imports = ImportMap::from_file(&syntax);
    let mut visitor = FactVisitor::new(&imports);
    visitor.visit_file(&syntax);
    visitor.operations
}

fn matching<'a>(
    facts: &'a [crate::source::SourceOperationFact],
    kind: SourceOperationKind,
    name: &str,
) -> Vec<&'a crate::source::SourceOperationFact> {
    facts
        .iter()
        .filter(|fact| fact.kind == kind && fact.identity.name == name)
        .collect()
}

#[path = "tests/visitor_typed_places.rs"]
mod visitor_typed_places;

#[path = "tests/visitor_field_truth.rs"]
mod visitor_field_truth;

#[path = "tests/visitor_remaining_authority.rs"]
mod visitor_remaining_authority;
