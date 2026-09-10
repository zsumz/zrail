//! Entry-set authority respects presence versus prohibition, without guessing disjoint relations.

use super::super::files_test::{configured, kinds, protected};
use crate::{ChangeKind, RepositoryEntryMode};

#[test]
fn inspection_scope_reduction_and_removal_require_protected_review() {
    let mut before = configured("kind = 'inspect'");
    before.repository.files[0].entry = RepositoryEntryMode::Any;
    assert!(kinds(&before, &before).is_empty());
    let mut expanded = before.clone();
    expanded.repository.files[0].include.push("docs/**".into());
    assert_eq!(kinds(&before, &expanded), [ChangeKind::Revoke]);
    assert_eq!(kinds(&expanded, &before), [ChangeKind::Grant]);
    protected(&expanded, &before);
    let mut excluded = before.clone();
    excluded.repository.files[0]
        .exclude
        .push("data/hidden/**".into());
    assert_eq!(kinds(&before, &excluded), [ChangeKind::Grant]);
    assert_eq!(kinds(&excluded, &before), [ChangeKind::Revoke]);
    protected(&before, &excluded);
    let mut changed = before.clone();
    changed.repository.files[0].include = vec!["data/*.rs".into()];
    assert_eq!(kinds(&before, &changed), [ChangeKind::Unknown]);
    protected(&before, &changed);
    for predicate in [
        "kind = 'count', minimum = 1",
        "kind = 'exact-paths', paths = []",
    ] {
        let mut changed = configured(predicate);
        changed.repository.files[0].entry = RepositoryEntryMode::Any;
        assert_eq!(kinds(&before, &changed), [ChangeKind::Unknown]);
        protected(&before, &changed);
        protected(&changed, &before);
    }
    let mut removed = before.clone();
    removed.repository.files.clear();
    assert_eq!(kinds(&before, &removed), [ChangeKind::Grant]);
    protected(&before, &removed);
}

#[test]
fn non_directory_selection_has_quantifier_aware_protected_diffs() {
    for (smaller, larger) in [
        (RepositoryEntryMode::File, RepositoryEntryMode::NonDirectory),
        (RepositoryEntryMode::NonDirectory, RepositoryEntryMode::Any),
    ] {
        for (predicate, forward, reverse) in [
            (
                "kind = 'count', minimum = 0, maximum = 0",
                ChangeKind::Revoke,
                ChangeKind::Grant,
            ),
            (
                "kind = 'count', minimum = 1",
                ChangeKind::Grant,
                ChangeKind::Revoke,
            ),
            (
                "kind = 'count', minimum = 1, maximum = 2",
                ChangeKind::Unknown,
                ChangeKind::Unknown,
            ),
            (
                "kind = 'exact-paths', paths = []",
                ChangeKind::Unknown,
                ChangeKind::Unknown,
            ),
        ] {
            let mut before = configured(predicate);
            before.repository.files[0].entry = smaller;
            let mut after = before.clone();
            after.repository.files[0].entry = larger;
            assert_eq!(kinds(&before, &after), [forward]);
            assert_eq!(kinds(&after, &before), [reverse]);
            if forward != ChangeKind::Revoke {
                protected(&before, &after);
            }
            if reverse != ChangeKind::Revoke {
                protected(&after, &before);
            }
        }
    }
}

#[test]
fn directories_and_non_directories_have_no_proven_subset_relation() {
    let mut before = configured("kind = 'count', minimum = 0, maximum = 0");
    before.repository.files[0].entry = RepositoryEntryMode::Directory;
    let mut after = before.clone();
    after.repository.files[0].entry = RepositoryEntryMode::NonDirectory;
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
    assert_eq!(kinds(&after, &before), [ChangeKind::Unknown]);
    protected(&before, &after);
    protected(&after, &before);
}
