//! Cargo manifest resolution indirection remains explicit.

use toml::Value;

use super::{CargoAuthorityKind, manifest, repository_configuration, root_cargo_config_path};

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
fn root_cargo_configuration_is_exact_fail_closed_authority() {
    for path in [".cargo/config", ".cargo/config.toml"] {
        assert!(root_cargo_config_path(path));
        let surface = repository_configuration(path);
        assert_eq!(surface.kind, CargoAuthorityKind::RepositoryConfiguration);
        assert_eq!(surface.path, path);
    }
    for near_miss in [".cargo/config.toml.bak", "cargo/config", ".cargo/config/"] {
        assert!(!root_cargo_config_path(near_miss));
    }
}

fn parse(source: &str) -> Value {
    source.parse().expect("parse Cargo fixture")
}
