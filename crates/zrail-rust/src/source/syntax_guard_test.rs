//! Compilation-domain guards combine and overlap without conflation.

use super::SyntaxGuard;

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
    assert_eq!(
        SyntaxGuard::Conditional.combine(SyntaxGuard::TestOnly),
        SyntaxGuard::ConditionalTestOnly
    );
    assert_eq!(
        SyntaxGuard::Conditional.combine(SyntaxGuard::ProductionOnly),
        SyntaxGuard::ConditionalProductionOnly
    );
    assert!(!SyntaxGuard::ConditionalTestOnly.is_exact());
    assert!(SyntaxGuard::ConditionalTestOnly.is_test_only());
    assert!(!SyntaxGuard::ConditionalTestOnly.overlaps(SyntaxGuard::ProductionOnly));
}
