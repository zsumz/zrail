//! Exact worlds reject feature-controlled target and source-graph attributes.

use super::file;

#[test]
fn detects_feature_gated_path_test_and_bench_attributes() {
    let syntax = syn::parse_file(
        r#"
#[cfg_attr(feature = "alternate", path = "alternate.rs")]
mod selected {}
#[cfg_attr(feature = "proof", test)]
fn proof() {}
#[cfg_attr(feature = "measure", bench)]
fn measure() {}
#[cfg_attr(feature = "docs", doc = "safe")]
fn documented() {}
"#,
    )
    .expect("parse fixture");

    assert_eq!(file(&syntax).len(), 3);
}
