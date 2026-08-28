//! Construction candidates retain namespace-aware generic-root provenance.

use crate::source::{GenericRootShadow, RootLookupNamespace, SourceOperationKind};

#[test]
fn type_and_const_generics_shadow_only_their_declared_namespace() {
    let facts = super::operations(
        r"
struct Marker;
enum Choice { Ready(u64) }
trait Factory { fn Ready(value: u64) -> Self; }
fn const_value<const Marker: usize>() -> usize { Marker }
fn associated<Choice: Factory>() -> Choice { Choice::Ready(1) }
fn bare_type<Marker>() { let _ = Marker; }
fn bare_prelude<Some>(value: u8) -> Option<u8> { Some(value) }
",
    );

    let marker_const = operation(&facts, "Marker", SourceOperationKind::ConstructorCapability);
    assert_eq!(marker_const.root_lookup, Some(RootLookupNamespace::Value));
    assert_eq!(
        marker_const.generic_shadow,
        Some(GenericRootShadow::ConstParameter)
    );

    let choice = operation(
        &facts,
        "Choice::Ready",
        SourceOperationKind::TypeConstruction,
    );
    assert_eq!(choice.root_lookup, Some(RootLookupNamespace::Type));
    assert_eq!(
        choice.generic_shadow,
        Some(GenericRootShadow::TypeParameter)
    );

    let bare_type = facts
        .iter()
        .find(|fact| {
            fact.identity.written.as_deref() == Some("Marker") && fact.generic_shadow.is_none()
        })
        .expect("bare type generic should not shadow a value constructor");
    assert_eq!(bare_type.root_lookup, Some(RootLookupNamespace::Value));

    let some = operation(&facts, "Some", SourceOperationKind::TypeConstruction);
    assert_eq!(some.root_lookup, Some(RootLookupNamespace::Value));
    assert_eq!(some.generic_shadow, None);
}

fn operation<'a>(
    facts: &'a [crate::source::SourceOperationFact],
    written: &str,
    kind: SourceOperationKind,
) -> &'a crate::source::SourceOperationFact {
    facts
        .iter()
        .find(|fact| fact.identity.written.as_deref() == Some(written) && fact.kind == kind)
        .unwrap_or_else(|| panic!("missing {kind:?} operation for {written}: {facts:#?}"))
}
