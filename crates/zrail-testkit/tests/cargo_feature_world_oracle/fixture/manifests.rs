//! Cargo and zrail manifests vary only the feature contribution under test.

use std::fmt::Write as _;

use super::{Family, Scenario};

pub(super) fn workspace(resolver: &str, scenario: Scenario) -> String {
    let members = scenario
        .members()
        .iter()
        .map(|member| format!("\"{member}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[workspace]\nmembers = [{members}]\nresolver = \"{resolver}\"\n")
}

pub(super) fn app(scenario: Scenario) -> String {
    let normal = feature_suffix(scenario.target_feature());
    let contextual = match scenario.family() {
        Family::Direct => format!(
            "{DIRECT_BUILD_HEAD}{} }}",
            feature_suffix(scenario.host_feature())
        ),
        Family::Transitive => TRANSITIVE_BUILD.into(),
        Family::ProcMacro => PROC_MACRO_DEPENDENCY.into(),
    };
    format!("{APP_HEAD}{normal}{APP_DEPENDENCY_END}{contextual}\n")
}

pub(super) fn helper(scenario: Scenario) -> String {
    let feature = feature_suffix(scenario.host_feature());
    format!("{HELPER_HEAD}{feature} }}\n")
}

pub(super) fn shared(scenario: Scenario) -> String {
    let default = matches!(scenario, Scenario::DefaultDirect)
        .then_some("default = [\"context\"]\n")
        .unwrap_or("");
    format!("{SHARED_HEAD}{default}context = []\n")
}

pub(super) fn contract(scenario: Scenario) -> String {
    let mut contract = CONTRACT.to_owned();
    let default_features = matches!(scenario, Scenario::DefaultDirect);
    let features = if matches!(scenario, Scenario::SelectedDirect) {
        "[\"context\"]"
    } else {
        "[]"
    };
    write!(
        contract,
        "\n[[source.rust.feature_worlds.packages]]\npackage = \"oracle-shared\"\ndefault_features = {default_features}\nfeatures = {features}\n"
    )
    .expect("write shared feature selection");
    for member in scenario.members().iter().skip(2) {
        write!(
            contract,
            "\n[[source.rust.feature_worlds.packages]]\npackage = \"oracle-{member}\"\ndefault_features = false\nfeatures = []\n"
        )
        .expect("write feature-world package");
    }
    contract
}

fn feature_suffix(enabled: bool) -> &'static str {
    if enabled {
        ", features = [\"context\"]"
    } else {
        ""
    }
}

const APP_HEAD: &str = r#"[package]
name = "oracle-app"
version = "0.0.0"
edition = "2024"

[dependencies]
oracle-shared = { path = "../shared", default-features = false"#;
const APP_DEPENDENCY_END: &str = " }\n";
const DIRECT_BUILD_HEAD: &str = r#"
[build-dependencies]
oracle-shared = { path = "../shared", default-features = false"#;
const TRANSITIVE_BUILD: &str = r#"
[build-dependencies]
oracle-helper = { path = "../helper", default-features = false }"#;
const PROC_MACRO_DEPENDENCY: &str = r#"oracle-macros = { path = "../macros" }"#;
const SHARED_HEAD: &str = r#"[package]
name = "oracle-shared"
version = "0.0.0"
edition = "2024"

[features]
"#;
const HELPER_HEAD: &str = r#"[package]
name = "oracle-helper"
version = "0.0.0"
edition = "2024"

[dependencies]
oracle-shared = { path = "../shared", default-features = false"#;
pub(super) const PROC_MACRO: &str = r#"[package]
name = "oracle-macros"
version = "0.0.0"
edition = "2024"

[lib]
proc-macro = true

[dependencies]
oracle-helper = { path = "../helper", default-features = false }
"#;
const CONTRACT: &str = include_str!("../contract.toml");
