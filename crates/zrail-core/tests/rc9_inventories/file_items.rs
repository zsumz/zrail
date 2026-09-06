//! Direct file-item inventories reject hidden scope changes and malformed written identities.

use super::{rule, validate};
use crate::RustInventorySubject;

#[test]
fn file_module_and_variant_subjects_require_complete_written_identifiers() {
    for (kind, field, valid, invalid) in [
        ("written-file-modules", "names", "r#tls", "outer::tls"),
        (
            "written-file-enum-variants",
            "variants",
            "r#Backend::r#Legacy",
            "crate::Backend::Legacy",
        ),
    ] {
        let mut selected = rule(&format!(
            "kind='exact-counts',counts=[{{path='src/lib.rs',name='{valid}',count=1}}]"
        ));
        selected.subject = toml::from_str(&format!("kind='{kind}'\n{field}=['{valid}']"))
            .expect("file item subject");
        validate(selected.clone()).expect("exact selected file item");
        selected.assertion = crate::RustInventoryAssertion::Count {
            minimum: 0,
            maximum: Some(0),
        };
        for value in [
            invalid,
            "",
            "*",
            "_",
            "Backend<T>::Legacy",
            "Backend::Legacy()",
        ] {
            selected.subject = toml::from_str(&format!("kind='{kind}'\n{field}=['{value}']"))
                .expect("typed selector");
            assert!(validate(selected.clone()).is_err(), "{kind} {value}");
        }
        for extra in [
            "recursive=true",
            "scope='all'",
            "visibility='public'",
            "resolve=true",
        ] {
            assert!(
                toml::from_str::<RustInventorySubject>(&format!(
                    "kind='{kind}'\n{field}=['{valid}']\n{extra}"
                ))
                .is_err()
            );
        }
    }
}
