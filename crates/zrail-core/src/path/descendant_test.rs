//! Prefix proofs follow the existing glob language, including zero-length `**` mounts.

use super::glob_can_match_descendant;

#[test]
fn exact_paths_do_not_imply_descendants() {
    for (pattern, prefix, expected) in [
        ("src/lib.rs", "src", true),
        ("src/lib.rs", "src/lib.rs", false),
        ("src/lib.rs", "src/child", false),
        ("src/**/*.rs", "target", false),
        ("src/**/*.rs", "src/.git", true),
        ("src/*/lib.rs", "src/one/two", false),
        ("**/.git/**", "src/nested/.git", true),
        ("**/*.rs", "one/two/three", true),
        ("**/src/*", "src", true),
        ("**/**/lib.rs", "one/two", true),
        ("", "", false),
        ("LICENSE", "", true),
        ("?/*.rs", "é", false),
        ("??/*.rs", "é", true),
    ] {
        assert_eq!(
            glob_can_match_descendant(pattern, prefix),
            Ok(expected),
            "{pattern:?} under {prefix:?}"
        );
    }
}

#[test]
fn every_concrete_descendant_match_has_a_live_prefix() {
    let components = ["src", "target", ".git", "x.rs"];
    let patterns = ["**", "**/*.rs", "*/**/x.rs", "src/*", "**/.git/**", "*/?"];
    for pattern in patterns {
        for first in components {
            for second in components {
                for third in components {
                    let path = format!("{first}/{second}/{third}");
                    if crate::glob_matches(pattern, &path) {
                        for prefix in [first.to_owned(), format!("{first}/{second}")] {
                            assert_eq!(glob_can_match_descendant(pattern, &prefix), Ok(true));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn oversized_prefix_queries_cannot_claim_disjointness() {
    assert!(glob_can_match_descendant(&"*".repeat(4097), "src").is_err());
    assert!(glob_can_match_descendant("**", &"x".repeat(4097)).is_err());
    assert!(glob_can_match_descendant("**", &"x/".repeat(257)).is_err());
}
