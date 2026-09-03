//! Parse diagnostics name both spellings of aliased macro authority fields.

use crate::contract::load::ContractFile;

use super::parse_error;

#[test]
fn duplicate_resolution_alias_names_the_canonical_and_legacy_keys() {
    let source = r#"
[source.rust.macros]
mode = "allow"

[[source.rust.macros.allow]]
name = "local::reviewed"
resolution = "exact"
binding = "exact"
reason = "The expansion has already been reviewed."
"#;
    let error = toml::from_str::<ContractFile>(source)
        .expect_err("both aliases must remain structurally invalid");
    let message = parse_error(&error);

    assert!(message.contains("`resolution` is canonical"), "{message}");
    assert!(
        message.contains("`binding` is its legacy alias"),
        "{message}"
    );
}
