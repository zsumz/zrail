//! Direct generic parameters cannot impersonate outer constructors.

use super::fixture::{Repository, construction_owner, exact_owner_count};

#[test]
fn const_generic_does_not_impersonate_unit_constructor() {
    let repository = Repository::new(
        "const-unit",
        "mod owner; pub struct Marker; pub fn real() -> Marker { Marker } #[allow(non_upper_case_globals)] pub fn read<const Marker: usize>() -> usize { Marker }",
        "pub fn own() -> crate::Marker { crate::Marker }",
        &construction_owner("marker-construction", "crate::Marker"),
    );
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "marker-construction", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_type_associated_function_does_not_impersonate_variant() {
    let repository = Repository::new(
        "generic-function",
        "mod owner; pub enum Choice { Ready(u64) } pub fn real() -> Choice { Choice::Ready(1) } pub trait Factory { #[allow(non_snake_case)] fn Ready(value: u64) -> Self; } pub fn make<Choice: Factory>() -> Choice { Choice::Ready(44) }",
        "pub fn own() -> crate::Choice { crate::Choice::Ready(1) }",
        &construction_owner("choice-construction", "crate::Choice"),
    );
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "choice-construction", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn generic_type_associated_const_does_not_impersonate_unit_variant() {
    let repository = Repository::new(
        "generic-const",
        "mod owner; pub enum State { Ready } pub fn real() -> State { State::Ready } pub trait Flag { #[allow(non_upper_case_globals)] const Ready: Self; } pub fn make<State: Flag>() -> State { State::Ready }",
        "pub fn own() -> crate::State { crate::State::Ready }",
        &construction_owner("state-construction", "crate::State"),
    );
    let report = repository.check();
    assert_eq!(
        exact_owner_count(&report, "state-construction", "src/lib.rs"),
        1,
        "{}",
        report.human()
    );
}
