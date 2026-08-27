//! Implicit source fields and nested item scopes preserve honest operation identity.

use zrail_core::AnalysisQuality;

use crate::source::SourceOperationKind;

use super::{matching, operations};

#[test]
fn destructuring_assignment_emits_source_member_reads() {
    let facts = operations(
        r"struct Vault { secret: String, spare: usize }
fn assign(vault: Vault, mut sink: String) {
    Vault { secret: sink, spare: _ } = vault;
}
",
    );

    assert_exact(&facts, 3, SourceOperationKind::FieldRead, "Vault::secret");
    assert!(matching(&facts, SourceOperationKind::FieldRead, "Vault::spare").is_empty());
    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.span.is_some_and(|span| span.line == 3)
    }));
}

#[test]
fn nested_destructuring_emits_each_source_projection() {
    let facts = operations(
        r"struct Inner { value: usize }
struct Outer { inner: Inner }
struct State { epoch: usize }
fn assign(source: Outer, state: &mut State) {
    Outer {
        inner: Inner {
            value: state.epoch,
        },
    } = source;
}
",
    );

    assert_exact(&facts, 6, SourceOperationKind::FieldRead, "Outer::inner");
    assert_exact(&facts, 7, SourceOperationKind::FieldRead, "Inner::value");
    assert_exact(&facts, 7, SourceOperationKind::FieldWrite, "State::epoch");
    assert!(matching(&facts, SourceOperationKind::FieldRead, "State::epoch").is_empty());
}

#[test]
fn struct_update_retains_deferred_receipt() {
    let facts = operations(
        r"struct State { public: usize, secret: usize, spare: usize }
fn update(previous: State) -> State {
    State {
        public: 10,
        ..previous
    }
}
fn opaque(previous: External) -> External {
    External { public: 10, ..previous }
}
",
    );

    let state = deferred_update(&facts, "State::*", 5);
    let update = state.struct_update.as_ref().expect("deferred update");
    assert_eq!(update.written, "State");
    assert_eq!(update.explicit_fields.len(), 1);
    assert_eq!(update.explicit_fields[0].name, "public");
    deferred_update(&facts, "External::*", 9);
}

#[test]
fn struct_update_receipt_tracks_explicit_field_cfg() {
    let facts = operations(
        r#"struct State {
    public: usize,
    #[cfg(feature = "extra")]
    extra: usize,
}
fn update(previous: State) -> State {
    State {
        #[cfg(feature = "direct")]
        public: 10,
        ..previous
    }
}
"#,
    );

    let receipt = deferred_update(&facts, "State::*", 10);
    let update = receipt.struct_update.as_ref().expect("deferred update");
    assert_eq!(update.explicit_fields.len(), 1);
    assert_eq!(update.explicit_fields[0].name, "public");
    assert_eq!(
        update.explicit_fields[0].guard.canonical_name(),
        "cfg:feature=\"direct\""
    );
}

#[test]
fn struct_update_defers_cfg_partitioned_declarations() {
    let facts = operations(
        r#"struct State {
    #[cfg(feature = "wide")]
    value: u64,
    #[cfg(not(feature = "wide"))]
    value: u32,
}
fn update(previous: State) -> State { State { ..previous } }
"#,
    );

    let receipt = deferred_update(&facts, "State::*", 7);
    assert!(
        receipt
            .struct_update
            .as_ref()
            .expect("deferred update")
            .explicit_fields
            .is_empty()
    );
}

#[test]
fn nested_item_does_not_inherit_outer_value_bindings() {
    let facts = operations(
        r"struct A { secret: usize }
struct B { secret: usize }
static value: B = B { secret: 0 };
fn outer(value: A) {
    fn inner() { let _ = value.secret; }
    inner();
}
",
    );

    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldRead
            && fact.identity.name == "A::secret"
            && fact.identity.quality == AnalysisQuality::Exact
            && fact.identity.span.is_some_and(|span| span.line == 5)
    }));
    assert!(facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::FieldRead
            && fact.identity.name == "<unresolved>::secret"
            && fact.identity.quality == AnalysisQuality::Unresolved
            && fact.identity.span.is_some_and(|span| span.line == 5)
    }));
}

#[test]
fn unit_struct_assignee_is_not_a_construction() {
    let facts = operations(
        r"struct Marker;
fn assign(incoming: Marker) { Marker = incoming; }
",
    );

    assert_no_construction(&facts, 2, "Marker");
}

#[test]
fn unit_variant_assignee_is_not_a_construction() {
    let facts = operations(
        r"enum Signal { Ready }
fn assign(incoming: Signal) { Signal::Ready = incoming; }
",
    );

    assert_no_construction(&facts, 2, "Signal::Ready");
}

fn assert_no_construction(facts: &[crate::source::SourceOperationFact], line: usize, name: &str) {
    assert!(!facts.iter().any(|fact| {
        fact.kind == SourceOperationKind::TypeConstruction
            && fact.identity.name == name
            && fact.identity.span.is_some_and(|span| span.line == line)
    }));
}

fn deferred_update<'a>(
    facts: &'a [crate::source::SourceOperationFact],
    name: &str,
    line: usize,
) -> &'a crate::source::SourceOperationFact {
    facts
        .iter()
        .find(|fact| {
            fact.kind == SourceOperationKind::FieldRead
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Unresolved
                && fact.identity.span.is_some_and(|span| span.line == line)
                && fact.struct_update.is_some()
        })
        .unwrap_or_else(|| panic!("missing deferred update {name} on line {line}: {facts:#?}"))
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
                && fact.identity.name == name
                && fact.identity.quality == AnalysisQuality::Exact
                && fact.identity.span.is_some_and(|span| span.line == line)
        }),
        "missing {kind:?} {name} on line {line}: {facts:#?}"
    );
}
