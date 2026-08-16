//! Cargo resolution indirection is distinguished from unrelated local configuration.

use toml::Value;

use super::{config_surfaces, manifest};

#[test]
fn manifest_patch_and_replace_tables_are_unsupported_surfaces() {
    let value = parse(
        r#"
[patch.crates-io]
uuid = { git = "https://example.test/uuid" }

[replace]
"old:1.0.0" = { path = "vendor/old" }
"#,
    );
    let mut overrides = Vec::new();

    manifest(&value, "Cargo.toml", &mut overrides);

    assert_eq!(overrides.len(), 2);
    assert!(overrides.iter().all(|item| item.path == "Cargo.toml"));
}

#[test]
fn config_source_paths_and_registry_mappings_are_unsupported() {
    let value = parse(
        r#"
paths = ["vendor"]

[source.crates-io]
replace-with = "mirror"

[registries.private]
index = "https://example.test/index"

[registry]
default = "private"
"#,
    );

    let surfaces = config_surfaces(&value);

    assert_eq!(surfaces.len(), 4);
    assert!(surfaces.contains("Cargo config paths override"));
    assert!(surfaces.contains("Cargo config source mapping or replacement"));
    assert!(surfaces.contains("Cargo config named registry mapping"));
    assert!(surfaces.contains("Cargo config default registry mapping"));
}

#[test]
fn config_includes_are_an_unattested_recursive_resolution_surface() {
    for include in [
        "include = [\"required.toml\"]",
        "include = [{ path = \"required.toml\" }]",
        "include = [{ path = \"optional.toml\", optional = true }]",
        "include = [\"../outside.toml\"]",
    ] {
        let value = include.parse::<toml::Value>().expect("parse include");
        assert!(
            config_surfaces(&value)
                .contains("Cargo configuration includes additional files whose effective resolution is not attested"),
            "missing include surface for {include}"
        );
    }
}

#[test]
fn build_and_network_configuration_do_not_claim_resolution_authority() {
    let value = parse(
        r#"
[build]
target-dir = "target"

[net]
offline = true

[registry]
global-credential-providers = ["cargo:token"]

[registries.private]
credential-provider = "cargo:token"
"#,
    );

    assert!(config_surfaces(&value).is_empty());
}

fn parse(source: &str) -> Value {
    source.parse().expect("parse Cargo fixture")
}
