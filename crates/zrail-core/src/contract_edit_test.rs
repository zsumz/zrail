//! Contract-source editing preserves authored narrative and layout.

use super::{format_contract_source, migrate_contract_source};

#[test]
fn migration_changes_only_schema_import_patterns_and_legacy_keys() {
    let source = r#"# Architecture narrative.
schema=1 # contract epoch
imports = [
  # >>> generated imports
  'policy/*.toml', # policy fragments
  "manual.toml",
]

[[source.rust.macros.allow]]
name = "generate"
"binding" = "conservative" # keep this explanation
bindings = "opaque"

# <<< generated imports
"#;
    let expected = r#"# Architecture narrative.
schema=2 # contract epoch
imports = [
  # >>> generated imports
  "policy/a.toml", "policy/b.toml", # policy fragments
  "manual.toml",
]

[[source.rust.macros.allow]]
name = "generate"
"resolution" = "conservative" # keep this explanation
namespace_effect = "opaque"

# <<< generated imports
"#;

    let migrated = migrate_contract_source(
        source,
        true,
        &[
            "manual.toml".into(),
            "policy/b.toml".into(),
            "policy/a.toml".into(),
        ],
    )
    .expect("migrate source");

    assert_eq!(migrated, expected);
    assert_eq!(
        migrated,
        migrate_contract_source(
            &migrated,
            true,
            &[
                "policy/a.toml".into(),
                "policy/b.toml".into(),
                "manual.toml".into(),
            ],
        )
        .expect("repeat migration")
    );
}

#[test]
fn formatting_preserves_comments_order_spacing_and_markers() {
    let source = concat!(
        "# Contract narrative\n",
        "schema=2 # authored spacing\n",
        "\n",
        "# >>> generated\n",
        "z=['last',  'still-last']\n",
        "a = 'first by design' # explanation\n",
        "# <<< generated",
    );
    let expected = format!("{source}\n");

    let formatted = format_contract_source(source).expect("format contract source");

    assert_eq!(formatted, expected);
    assert_eq!(
        formatted,
        format_contract_source(&formatted).expect("repeat formatting")
    );
    assert!(format_contract_source("schema = [").is_err());
}

#[test]
fn migration_preserves_inline_table_spellings() {
    let source = r#"schema = 1

[source.rust]
item_macros = [{ name = "item", binding = "exact" }] # inline item policy

[source.rust.macros]
allow = [{ name = "derive", binding = "conservative", bindings = "opaque" }]
"#;
    let expected = source
        .replace("schema = 1", "schema = 2")
        .replace("binding =", "resolution =")
        .replace("bindings =", "namespace_effect =");

    let migrated = migrate_contract_source(source, true, &[]).expect("migrate inline tables");

    assert_eq!(migrated, expected);
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
