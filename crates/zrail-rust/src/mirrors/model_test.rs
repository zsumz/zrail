//! Mirror policy identities remain unambiguous for every valid repository path.

use super::policy_id_fields;

#[test]
fn length_prefixed_identity_distinguishes_delimiter_bearing_paths() {
    let left = ("src/a.rs", "tests/b.rs::c.rs", "proof");
    let right = ("src/a.rs::tests/b.rs", "c.rs", "proof");
    assert_eq!(
        format!("{}::{}::{}", left.0, left.1, left.2),
        format!("{}::{}::{}", right.0, right.1, right.2),
    );
    assert_ne!(
        policy_id_fields(left.0, left.1, left.2),
        policy_id_fields(right.0, right.1, right.2),
    );
}
