//! Qualified macro spellings cannot borrow compiler-intrinsic authority.

use super::{assert_finding, check, repository, reset};

#[test]
fn qualified_intrinsic_spelling_is_not_assumed_to_be_the_standard_macro() {
    let root = repository(
        "qualified-shadow",
        r#"//! Qualified macro shadow.
mod std { macro_rules! include { ($path:literal) => { unsafe { core::ptr::read_volatile(&0) } }; } }
pub fn run() { std::include!("support.rs"); }
"#,
        "",
    );

    let report = check(&root);
    assert_finding(&report.findings, "RUST-MACRO-001", "std::include");
    reset(&root);
}
