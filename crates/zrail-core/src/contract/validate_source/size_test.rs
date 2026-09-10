//! Scoped size contracts have closed syntax and bounded, actionable authority.

use crate::{SizePolicyContract, contract::validate_fixture_test::minimal_contract};

use super::{ValidationErrors, validate};

const SCOPE: &str = r#"
[[overrides]]
name = "core"
include = ["crates/core/src/**"]
packages = ["core"]
roles = ["implementation", "test"]
budget = { target = 240, soft = 300, hard = 500 }
reason = "Keep the core reviewable."
"#;

#[test]
fn syntax_rejects_unknown_fields_modes_and_duplicate_keys() {
    let policy: SizePolicyContract = toml::from_str(SCOPE).expect("strict policy");
    assert_eq!(
        policy.overrides[0].budget.target_mode,
        crate::SizeTargetMode::Error
    );
    for invalid in [
        SCOPE.replace("target = 240", "target = 240, target = 250"),
        SCOPE.replace("target = 240", "target = 240, target_mode = 'ignore'"),
        SCOPE.replace("hard = 500", "hard = 500, force = true"),
        format!("{SCOPE}unknown = true\n"),
        SCOPE.replace("implementation", "anything"),
    ] {
        assert!(
            toml::from_str::<SizePolicyContract>(&invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn selectors_limits_names_and_reasons_are_validated() {
    assert!(errors(SCOPE).is_empty());
    for (invalid, message) in [
        (
            SCOPE.replace("target = 240", "target = 0"),
            "0 < target <= hard",
        ),
        (
            SCOPE.replace("soft = 300", "soft = 501"),
            "target <= soft <= hard",
        ),
        (
            SCOPE.replace("soft = 300", "soft = 239"),
            "target <= soft <= hard",
        ),
        (
            SCOPE.replace("Keep the core reviewable.", " "),
            "requires a reason",
        ),
        (
            SCOPE.replace("crates/core/src/**", "../src/**"),
            "not canonical",
        ),
        (
            SCOPE.replace("[\"core\"]", "[\"core\", \"core\"]"),
            "duplicate selector",
        ),
        (
            SCOPE.replace("[\"crates/core/src/**\"]", "[]"),
            "requires an include",
        ),
    ] {
        let result = errors(&invalid);
        assert!(result.contains(message), "{message}: {result}");
    }
}

#[test]
fn tiers_are_not_resolved_using_toml_order() {
    let same_selector = SCOPE
        .replace("name = \"core\"", "name = \"other\"")
        .replace(
            "[\"implementation\", \"test\"]",
            "[\"test\", \"implementation\"]",
        );
    assert!(errors(&format!("{SCOPE}{same_selector}")).contains("same-tier selector"));
    assert!(errors(&format!("{SCOPE}{SCOPE}")).contains("unique nonempty name"));
}

#[test]
fn exceptions_require_exact_paths_bounds_and_actionable_metadata() {
    let allowance = r#"
[[exceptions]]
path = "src/owner.rs"
hard = 600
reason = "Split the state-machine surface."
owner = "architecture"
issue = "ARCH-42"
"#;
    assert!(errors(allowance).is_empty());
    assert!(
        errors(&allowance.replace(
            "owner = \"architecture\"\nissue = \"ARCH-42\"",
            "tracking = 'readability'"
        ))
        .is_empty()
    );
    for (invalid, message) in [
        (
            allowance.replace("src/owner.rs", "src/*.rs"),
            "exact repository path",
        ),
        (
            allowance.replace("hard = 600", "hard = 0"),
            "positive hard bound",
        ),
        (
            allowance.replace("issue = \"ARCH-42\"", ""),
            "owner/issue pair",
        ),
        (
            allowance.replace("owner = \"architecture\"", "owner = ''"),
            "empty owner",
        ),
        (
            format!("{allowance}{allowance}"),
            "duplicate size exception",
        ),
    ] {
        let result = errors(&invalid);
        assert!(result.contains(message), "{message}: {result}");
    }
}

#[test]
fn required_metadata_forms_cannot_be_exchanged_to_bypass_a_consumer_policy() {
    let tracking = "[[exceptions]]\npath = 'src/owner.rs'\nhard = 600\nreason = 'Split this seam.'\ntracking = 'readability'\n";
    assert!(errors(&format!("exception_metadata = 'tracking'\n{tracking}")).is_empty());
    assert!(
        errors(&format!("exception_metadata = 'owner-issue'\n{tracking}"))
            .contains("lacks required")
    );
    let both = format!("{tracking}owner = 'architecture'\nissue = 'ARCH-42'\n");
    assert!(
        errors(&format!(
            "exception_metadata = 'owner-issue-and-tracking'\n{both}"
        ))
        .is_empty()
    );
}

#[test]
fn formatting_preserves_authored_budget_comments_and_layout() {
    let source = format!("# Reviewed threshold rationale\n{SCOPE}\n# Keep this tracking record\n");
    assert_eq!(
        crate::format_contract_source(&source).expect("format"),
        source
    );
}

fn errors(source: &str) -> String {
    let mut contract = minimal_contract();
    contract.source.rust.budgets = Some(toml::from_str(source).expect("policy syntax"));
    let mut errors = ValidationErrors::new();
    validate(&contract, &mut errors);
    errors.finish().join("\n")
}
