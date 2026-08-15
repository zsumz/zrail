//! Starter contract rendering preserves strict test and size defaults.

use super::render;
use crate::app::args::InitPreset;
use zrail_rust::BaselinePlan;

#[test]
fn roots_are_toml_escaped_and_defaults_remain_strict() {
    let contract = render(
        &["components/core".into(), "tools/quoted\"name".into()],
        InitPreset::Zsumz,
        &BaselinePlan::empty(),
    );

    assert!(contract.contains("roots = [\"components/core\", \"tools/quoted\\\"name\"]"));
    assert!(contract.contains("entrypoints = \"allow\""));
    assert!(contract.contains("tests = \"sibling\""));
    assert_eq!(contract.matches("hard = 300").count(), 4);
    assert_eq!(contract.matches("target = 300").count(), 4);
}

#[test]
fn rust_preset_keeps_conventional_tests_without_inventing_a_size_limit() {
    let contract = render(&[".".into()], InitPreset::Rust, &BaselinePlan::empty());

    assert!(contract.contains("tests = \"allow\""));
    assert!(!contract.contains("[source.rust.size"));
    assert!(!contract.contains("[[ratchet]]"));
}
