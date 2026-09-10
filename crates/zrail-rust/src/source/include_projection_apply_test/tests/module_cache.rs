//! Immutable module identity lookup reuses only completed, context-exact work.

use super::*;
use crate::source::{SourceInstanceId, include_projection_budget::ProjectionBudget};

#[test]
fn repeated_module_identity_queries_reuse_completed_work() {
    let index = fixture_index();
    let bindings = bindings(&index);
    let mut budget = ProjectionBudget::new(ProjectionLimits {
        work: 100,
        projected_facts: 0,
    });
    let first = bindings
        .effective_module(SourceInstanceId(0), &[], &mut budget)
        .unwrap();
    let used = budget.used_work();
    assert!(used > 0);
    assert_eq!(
        bindings
            .effective_module(SourceInstanceId(0), &[], &mut budget)
            .unwrap(),
        first
    );
    assert_eq!(budget.used_work(), used);
    bindings
        .effective_module(SourceInstanceId(1), &[], &mut budget)
        .unwrap();
    assert!(
        budget.used_work() > used,
        "include occurrences retain their own cache keys"
    );
}

#[test]
fn exhausted_work_is_not_cached_and_each_projection_pass_recounts_work() {
    let mut index = fixture_index();
    let bindings = bindings(&index);
    let mut empty = ProjectionBudget::new(ProjectionLimits {
        work: 0,
        projected_facts: 0,
    });
    assert!(
        bindings
            .effective_module(SourceInstanceId(0), &[], &mut empty)
            .is_err()
    );
    assert!(bindings.module_cache.borrow().is_empty());
    let limits = ProjectionLimits {
        work: 1_000,
        projected_facts: 10,
    };
    assert!(bindings.apply_with_limits(&mut index, limits).is_empty());
    let used = index.analysis_metrics.projection_work;
    let mut repeated = fixture_index();
    assert!(bindings.apply_with_limits(&mut repeated, limits).is_empty());
    assert_eq!(repeated.analysis_metrics.projection_work, used);
}

#[test]
fn lexical_floor_reuses_completed_work_without_crossing_scope_or_instance_boundaries() {
    let mut index = fixture_index();
    index.files[0] = parsed_file(
        "src/lib.rs",
        "mod outer { mod inner { fn run() { consume(); } } }",
    );
    let scope = &index.files[0].calls[0].lexical_scope;
    let bindings = bindings(&index);
    let mut budget = ProjectionBudget::new(ProjectionLimits {
        work: 100,
        projected_facts: 0,
    });
    let id = SourceInstanceId(0);
    let floor = bindings.lexical_floor(id, scope, &mut budget).unwrap();
    assert!(floor > 0);
    assert!(floor < scope.len());
    let used = budget.used_work();
    assert!(used > 0);
    assert_eq!(bindings.lexical_floor(id, scope, &mut budget), Ok(floor));
    assert_eq!(budget.used_work(), used);
    assert!(
        bindings
            .lexical_floor(id, &scope[..floor - 1], &mut budget)
            .unwrap()
            < floor
    );
    assert_eq!(
        bindings.lexical_floor(SourceInstanceId(1), scope, &mut budget),
        Ok(0)
    );
    assert!(budget.used_work() > used);
}

#[test]
fn exhausted_lexical_floor_queries_are_retried_and_each_pass_clears_cached_floors() {
    let mut index = fixture_index();
    index.files[0] = parsed_file("src/lib.rs", "mod inner { fn run() { consume(); } }");
    let scope = index.files[0].calls[0].lexical_scope.clone();
    let bindings = bindings(&index);
    let mut empty = ProjectionBudget::new(ProjectionLimits {
        work: 0,
        projected_facts: 0,
    });
    assert!(
        bindings
            .lexical_floor(SourceInstanceId(0), &scope, &mut empty)
            .is_err()
    );
    assert!(bindings.lexical_floor_cache.borrow().is_empty());
    let mut enough = ProjectionBudget::new(ProjectionLimits {
        work: 100,
        projected_facts: 0,
    });
    assert!(
        bindings
            .lexical_floor(SourceInstanceId(0), &scope, &mut enough)
            .unwrap()
            > 0
    );
    assert!(!bindings.lexical_floor_cache.borrow().is_empty());
    bindings.apply_with_limits(
        &mut index,
        ProjectionLimits {
            work: 0,
            projected_facts: 0,
        },
    );
    assert!(bindings.lexical_floor_cache.borrow().is_empty());
}
