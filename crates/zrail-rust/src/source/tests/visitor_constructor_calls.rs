//! Wrapped constructor callees retain one tuple-construction operation.

use crate::source::{ConstructorForm, SourceOperationKind};

use super::operations;

#[test]
fn parenthesized_constructor_alias_is_recorded_as_one_tuple_construction() {
    let facts = operations(
        r"
struct Ticket(usize);
use Ticket as make;
fn build() { let _ = (make)(1); }
",
    );
    let constructions = facts
        .iter()
        .filter(|fact| fact.kind == SourceOperationKind::TypeConstruction)
        .collect::<Vec<_>>();

    assert_eq!(constructions.len(), 1, "{constructions:#?}");
    assert_eq!(constructions[0].identity.written.as_deref(), Some("make"));
    assert_eq!(constructions[0].construction, Some(ConstructorForm::Tuple));
}
