//! Contract import headers retain strict list typing without requiring a full contract.

use super::contract_imports;

#[test]
fn imports_are_extracted_from_complete_or_fragment_contracts() {
    let imports = contract_imports(
        "schema = 1\nimports = ['zrail.d/*.toml', 'architecture/custom.rules']\n",
        "zrail.toml",
    )
    .expect("parse imports");

    assert_eq!(imports, ["zrail.d/*.toml", "architecture/custom.rules"]);
}

#[test]
fn malformed_import_lists_fail_before_snapshot_materialization() {
    let error = contract_imports("imports = 'zrail.d/base.toml'", "zrail.toml")
        .expect_err("imports must be a string array");

    assert!(error.to_string().contains("zrail.toml"));
}
