//! Starter contract rendering preserves strict test and size defaults.

use super::render;
use zrail_rust::BaselinePlan;

#[test]
fn roots_are_toml_escaped_and_defaults_remain_strict() {
    let contract = render(
        &["components/core".into(), "tools/quoted\"name".into()],
        &BaselinePlan::strict(),
    );

    assert!(contract.contains("roots = [\"components/core\", \"tools/quoted\\\"name\"]"));
    assert!(contract.contains("entrypoints = \"allow\""));
    assert!(contract.contains("tests = \"sibling\""));
    assert_eq!(contract.matches("hard = 300").count(), 4);
    assert_eq!(contract.matches("target = 300").count(), 4);
}
