//! Nested `cfg_attr` presence is reduced exactly for Cargo feature worlds.

use std::collections::BTreeSet;

use syn::ItemFn;

use super::cfg_guard;
use crate::source::{CompilationDomain, CompilationMode, GuardAvailability};

#[test]
fn feature_controlled_nested_cfg_is_exact_in_each_world() {
    let item =
        syn::parse_str::<ItemFn>("#[cfg_attr(feature = \"strict\", cfg(any()))] fn selected() {}")
            .expect("parse function");
    let guard = cfg_guard(&item.attrs);

    assert_eq!(
        guard.availability_in_domain(&domain("strict", ["strict"])),
        GuardAvailability::Absent
    );
    assert_eq!(
        guard.availability_in_domain(&domain("portable", [])),
        GuardAvailability::Exact
    );
}

#[test]
fn recursively_nested_cfg_attrs_control_item_presence_exactly() {
    let item = syn::parse_str::<ItemFn>(concat!(
        "#[cfg_attr(feature = \"outer\", ",
        "cfg_attr(feature = \"inner\", cfg(any())))] ",
        "fn selected() {}",
    ))
    .expect("parse nested function");
    let guard = cfg_guard(&item.attrs);

    assert_eq!(
        guard.availability_in_domain(&domain("none", [])),
        GuardAvailability::Exact
    );
    assert_eq!(
        guard.availability_in_domain(&domain("outer", ["outer"])),
        GuardAvailability::Exact
    );
    assert_eq!(
        guard.availability_in_domain(&domain("nested", ["inner", "outer"])),
        GuardAvailability::Absent
    );
}

#[test]
fn malformed_nested_cfg_attr_remains_possible_instead_of_becoming_inert() {
    let item = syn::parse_str::<ItemFn>(
        "#[cfg_attr(feature = \"outer\", cfg_attr(test))] fn selected() {}",
    )
    .expect("parse permissive nested function");

    assert_eq!(
        cfg_guard(&item.attrs).availability_in_domain(&domain("none", [])),
        GuardAvailability::Possible
    );
}

fn domain<const N: usize>(name: &str, features: [&str; N]) -> CompilationDomain {
    CompilationDomain {
        package: "app".into(),
        edition: "2024".into(),
        target: "app".into(),
        mode: CompilationMode::Library,
        feature_world: Some(name.into()),
        active_features: features
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>(),
    }
}
