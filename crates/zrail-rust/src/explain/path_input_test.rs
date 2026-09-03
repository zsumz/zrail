//! Path-input helper examples.

use super::edit_distance;

#[test]
fn edit_distance_handles_insertions_deletions_and_substitutions() {
    assert_eq!(edit_distance("src/lib.rs", "src/lib.rs"), 0);
    assert_eq!(edit_distance("src/lb.rs", "src/lib.rs"), 1);
    assert_eq!(edit_distance("src/libs.rs", "src/lib.rs"), 1);
    assert_eq!(edit_distance("src/lab.rs", "src/lib.rs"), 1);
}
