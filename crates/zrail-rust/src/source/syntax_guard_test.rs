//! Compilation-domain guards combine and overlap without conflation.

use super::{CfgPredicate, GuardAvailability, SyntaxGuard};

#[test]
fn ordinary_occurrences_overlap_both_compilation_domains() {
    assert!(SyntaxGuard::Ordinary.overlaps(SyntaxGuard::TestOnly));
    assert!(SyntaxGuard::Ordinary.overlaps(SyntaxGuard::ProductionOnly));
}

#[test]
fn test_and_production_only_occurrences_do_not_overlap() {
    assert!(!SyntaxGuard::TestOnly.overlaps(SyntaxGuard::ProductionOnly));
    assert!(!SyntaxGuard::Never.overlaps(SyntaxGuard::Ordinary));
}

#[test]
fn conditional_guards_preserve_test_and_production_intersections() {
    let conditional = SyntaxGuard::from_predicate(CfgPredicate::Opaque("unknown".into()));
    let test = conditional.combine(SyntaxGuard::TestOnly);
    let production = conditional.combine(SyntaxGuard::ProductionOnly);

    assert_eq!(test.canonical_name(), "cfg:all(test,opaque(unknown))");
    assert_eq!(
        production.canonical_name(),
        "cfg:all(opaque(unknown),not(test))"
    );
    assert!(!test.is_exact());
    assert!(test.is_test_only());
    assert!(!test.overlaps(SyntaxGuard::ProductionOnly));
}

#[test]
fn conditional_guards_are_possible_only_in_overlapping_domains() {
    let conditional = SyntaxGuard::from_predicate(CfgPredicate::Opaque("unknown".into()));
    let conditional_test = conditional.combine(SyntaxGuard::TestOnly);
    assert_eq!(
        conditional.availability_in(SyntaxGuard::Ordinary),
        GuardAvailability::Possible
    );
    assert!(conditional.available_in(SyntaxGuard::Ordinary));
    assert_eq!(
        conditional_test.availability_in(SyntaxGuard::TestOnly),
        GuardAvailability::Possible
    );
    assert_eq!(
        conditional_test.availability_in(SyntaxGuard::Ordinary),
        GuardAvailability::Absent
    );
    assert_eq!(
        SyntaxGuard::ProductionOnly.availability_in(SyntaxGuard::Ordinary),
        GuardAvailability::Exact
    );
}

#[test]
fn ordinary_binding_is_exact_under_conditional_occurrence() {
    let occurrence = SyntaxGuard::from_predicate(CfgPredicate::Opaque("unix".into()));

    assert_eq!(
        SyntaxGuard::Ordinary.availability_in(&occurrence),
        GuardAvailability::Exact
    );
}

#[test]
fn stronger_occurrence_makes_weaker_binding_exact() {
    let unix = CfgPredicate::Opaque("unix".into());
    let linux = CfgPredicate::Opaque("target_os=linux".into());
    let binding = SyntaxGuard::from_predicate(unix.clone());
    let occurrence = SyntaxGuard::from_predicate(CfgPredicate::all(vec![unix, linux]));

    assert_eq!(
        binding.availability_in(&occurrence),
        GuardAvailability::Exact
    );
}
