//! Exact top-level item identities retain quantifier-aware protected review.

use super::{contract, kinds};
use crate::ChangeKind;

#[test]
fn file_item_bans_and_required_quantities_review_selected_names_by_quantifier() {
    for (kind, field, old, extra) in [
        ("written-file-modules", "names", "tls", "plaintext"),
        (
            "written-file-enum-variants",
            "variants",
            "Backend::Legacy",
            "Backend::Retired",
        ),
    ] {
        for (assertion, expansion) in [
            ("kind='count',maximum=0", ChangeKind::Revoke),
            ("kind='count',minimum=1", ChangeKind::Grant),
        ] {
            let mut before = contract(assertion);
            before.source.rust.inventories[0].subject =
                toml::from_str(&format!("kind='{kind}'\n{field}=['{old}']")).expect("subject");
            let mut after = before.clone();
            after.source.rust.inventories[0].subject =
                toml::from_str(&format!("kind='{kind}'\n{field}=['{old}','{extra}']"))
                    .expect("expanded subject");
            assert_eq!(kinds(&before, &after), [expansion]);
            after.source.rust.inventories.clear();
            assert_eq!(kinds(&before, &after), [ChangeKind::Grant]);
        }
    }
    let mut before = contract("kind='exact-owners',owners=[]");
    before.source.rust.inventories[0].subject =
        toml::from_str("kind='written-file-modules'\nnames=['tls']").expect("module subject");
    let mut after = before.clone();
    after.source.rust.inventories[0].subject =
        toml::from_str("kind='written-paths-containing'\nnames=['tls']").expect("path subject");
    assert_eq!(kinds(&before, &after), [ChangeKind::Unknown]);
}
