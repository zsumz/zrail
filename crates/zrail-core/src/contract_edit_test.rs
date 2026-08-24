//! Contract-source editing remains structured and deterministic.

use super::{format_contract_source, migrate_contract_source};

#[test]
fn migration_replaces_patterns_and_legacy_macro_keys() {
    let source = r#"schema = 1
imports = ["policy/*.toml"]

[[source.rust.macros.allow]]
name = "generate"
binding = "conservative"
bindings = "opaque"
"#;

    let migrated = migrate_contract_source(
        source,
        true,
        &["policy/b.toml".into(), "policy/a.toml".into()],
    )
    .expect("migrate source");

    assert!(migrated.contains("schema = 2"));
    assert!(migrated.contains("\"policy/a.toml\""));
    assert!(migrated.contains("\"policy/b.toml\""));
    assert!(migrated.contains("resolution = \"conservative\""));
    assert!(migrated.contains("namespace_effect = \"opaque\""));
    assert!(!migrated.contains("binding ="));
    assert_eq!(
        migrated,
        migrate_contract_source(
            &migrated,
            true,
            &["policy/a.toml".into(), "policy/b.toml".into()]
        )
        .expect("repeat migration")
    );
}

#[test]
fn formatting_is_idempotent_and_rejects_invalid_toml() {
    let formatted =
        format_contract_source("schema=2\nadapters=['rust']").expect("format contract source");

    assert!(formatted.ends_with('\n'));
    assert_eq!(
        formatted,
        format_contract_source(&formatted).expect("repeat formatting")
    );
    assert!(format_contract_source("schema = [").is_err());
}

#[test]
fn migration_rejects_mixed_legacy_and_current_keys() {
    let error = migrate_contract_source(
        r#"[[source.rust.item_macros]]
name = "generate"
binding = "exact"
resolution = "exact"
"#,
        false,
        &[],
    )
    .expect_err("mixed keys must fail");

    assert!(error.to_string().contains("may not combine legacy"));
}
