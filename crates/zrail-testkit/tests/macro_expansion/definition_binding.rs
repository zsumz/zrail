//! Namespace authority requires the exact observed local macro definition.

use std::fs;

use zrail_core::ReportStatus;

use super::{build_lock, check, repository, reset};

#[test]
fn external_qualified_definition_binds_its_exact_file() {
    let root = repository(
        "external-qualified-definition",
        "//! Qualified definition fixture.\nmod local;\npub fn run() { let _ = local::reviewed!(); }\n",
        r#"
[[source.rust.macros.allow]]
name = "local::reviewed"
definition = "src/local.rs"
reason = "The exact local definition expands to one integer literal."
"#,
    );
    fs::write(
        root.join("src/local.rs"),
        "//! Reviewed macro.\nmacro_rules! reviewed { () => { 1 }; }\npub(crate) use reviewed;\n",
    )
    .expect("write reviewed macro definition");
    build_lock(&root, "zrail.toml".as_ref())
        .expect("build exact definition lock")
        .write(&root.join("zrail.lock"))
        .expect("write exact definition lock");

    let report = check(&root);

    assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
    reset(&root);
}

#[test]
fn same_package_same_leaf_definition_cannot_lend_binding_authority() {
    let root = repository(
        "wrong-qualified-definition",
        concat!(
            "//! Qualified local macro fixture.\n",
            "mod benign;\n",
            "mod evil;\n\n",
            "evil::inject!();\n\n",
            "pub fn hidden() {\n",
            "    let _ = Spawn(\"input\");\n",
            "}\n",
        ),
        r#"
[[source.rust.macros.allow]]
name = "evil::inject"
definition = "src/benign.rs"
bindings = "none"
reason = "Only the benign definition preserves the ordinary namespace."
"#,
    );
    fs::write(
        root.join("src/benign.rs"),
        "//! Benign macro.\nmacro_rules! inject { () => {}; }\npub(crate) use inject;\n",
    )
    .expect("write benign macro definition");
    fs::write(
        root.join("src/evil.rs"),
        concat!(
            "//! Mutating macro.\n",
            "macro_rules! inject { () => { use std::fs::read as Spawn; }; }\n",
            "pub(crate) use inject;\n",
        ),
    )
    .expect("write mutating macro definition");

    let report = check(&root);

    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-006"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.message.contains("evil::inject")
                && finding.message.contains("src/benign.rs")
                && finding.message.contains("src/evil.rs")
        }),
        "{}",
        report.human()
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002" && finding.path.as_deref() == Some("src/lib.rs")
        }),
        "{}",
        report.human()
    );
    reset(&root);
}

#[test]
fn test_definition_cannot_borrow_production_definition_binding_authority() {
    let root = repository(
        "wrong-cfg-definition",
        concat!(
            "//! Cfg definition fixture.\n",
            "#[cfg(test)]\n",
            "include!(\"test_macro.rs\");\n",
            "#[cfg(not(test))]\n",
            "include!(\"prod_macro.rs\");\n\n",
            "#[cfg(test)]\n",
            "choose!();\n\n",
            "#[cfg(test)]\n",
            "pub fn hidden() {\n",
            "    let _ = Spawn(\"input\");\n",
            "}\n",
        ),
        r#"
[[source.rust.macros.allow]]
name = "choose"
definition = "src/prod_macro.rs"
bindings = "none"
reason = "Only the production definition preserves the ordinary namespace."
"#,
    );
    fs::write(
        root.join("src/test_macro.rs"),
        concat!(
            "/// Test macro definition.\n",
            "macro_rules! choose {\n",
            "    () => { use std::fs::read as Spawn; };\n",
            "}\n",
        ),
    )
    .expect("write test macro definition");
    fs::write(
        root.join("src/prod_macro.rs"),
        "/// Production macro definition.\nmacro_rules! choose { () => {}; }\n",
    )
    .expect("write production macro definition");

    let report = check(&root);

    assert_rejected_with_opacity(&report, "choose");
    reset(&root);
}

#[test]
fn feature_definition_cannot_borrow_complement_definition_binding_authority() {
    let root = repository(
        "wrong-feature-definition",
        concat!(
            "//! Feature definition fixture.\n",
            "include!(\"definitions.rs\");\n\n",
            "#[cfg(feature = \"mutating\")]\n",
            "pub fn hidden() {\n",
            "    choose!();\n",
            "    let _ = Spawn(\"input\");\n",
            "}\n",
        ),
        r#"
[[source.rust.macros.allow]]
name = "include"
binding = "conservative"
reason = "Literal repository includes are source-graph inspected."

[[source.rust.macros.allow]]
name = "choose"
definition = "src/benign.rs"
bindings = "none"
reason = "Only the complementary definition preserves the ordinary namespace."
"#,
    );
    fs::write(
        root.join("Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"fixture\"\n",
            "version = \"0.0.0\"\n",
            "edition = \"2024\"\n\n",
            "[features]\n",
            "mutating = []\n",
        ),
    )
    .expect("declare feature fixture");
    fs::write(
        root.join("src/definitions.rs"),
        concat!(
            "/// Feature-selected definitions.\n",
            "#[cfg(feature = \"mutating\")]\n",
            "include!(\"evil.rs\");\n",
            "#[cfg(not(feature = \"mutating\"))]\n",
            "include!(\"benign.rs\");\n",
        ),
    )
    .expect("write feature selector");
    fs::write(
        root.join("src/evil.rs"),
        concat!(
            "/// Mutating feature definition.\n",
            "macro_rules! choose {\n",
            "    () => { use std::fs::read as Spawn; };\n",
            "}\n",
        ),
    )
    .expect("write mutating feature definition");
    fs::write(
        root.join("src/benign.rs"),
        "/// Benign complementary definition.\nmacro_rules! choose { () => {}; }\n",
    )
    .expect("write complementary feature definition");

    let report = check(&root);

    assert_rejected_with_opacity(&report, "choose");
    reset(&root);
}

fn assert_rejected_with_opacity(report: &zrail_core::Report, name: &str) {
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-MACRO-006"
                && finding.path.as_deref() == Some("src/lib.rs")
                && finding.message.contains(name)
        }),
        "{}",
        report.human()
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.id == "RUST-INCLUDE-002" && finding.path.as_deref() == Some("src/lib.rs")
        }),
        "{}",
        report.human()
    );
}
