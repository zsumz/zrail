//! Stable SHA-256 rendering examples.

use super::sha256_hex;

#[test]
fn digest_is_lowercase_fixed_width_hex() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
