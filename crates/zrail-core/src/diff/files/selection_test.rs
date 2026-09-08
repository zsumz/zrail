//! Entry-set authority respects presence versus prohibition, without guessing disjoint relations.

use super::super::files_test::{configured, kinds, protected};
use crate::{ChangeKind, RepositoryEntryMode};

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
