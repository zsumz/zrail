//! Direct-file predicates compare original detectors with complete native written quantities.

#[path = "legacy.rs"]
mod legacy;
#[path = "origins.rs"]
mod origins;

use super::native;
use std::collections::BTreeMap;

#[test]
fn frozen_file_module_predicate_distinguishes_declarations_from_nested_and_opaque_text() {
    let cases = [
        ("", 0),
        ("mod tls;", 1),
        ("mod tls {}", 1),
        ("pub mod tls {}", 1),
        ("#[cfg(any())] mod tls;", 1),
        ("mod outer { mod tls; }", 0),
        ("fn f() { mod tls {} }", 0),
        ("use p::tls;", 0),
        ("type tls = State;", 0),
        ("mod r#tls;", 0),
        ("mod TLS;", 0),
        ("mod tls; mod tls {}", 2),
        ("macro_rules! hidden { () => { mod tls; }; }", 0),
        ("const TEXT: &str = \"mod tls;\"; // mod tls;", 0),
        ("#[doc={ mod tls; \"\" }] struct X;", 0),
        ("mod current; mod plaintext; mod broker_set;", 2),
        ("#[cfg(test)] mod tls; #[cfg(not(test))] mod tls;", 2),
        ("mod resource; mod poller; mod tcp; mod timer;", 4),
    ];
    let subject = "kind='written-file-modules',names=['broker_set','plaintext','poller','resource','tcp','timer','tls']";
    let root = root("modules");
    for (source, quantity) in cases {
        let original = legacy::modules(source);
        assert_eq!(original.len(), quantity, "independent quantity: {source}");
        let mut expected = BTreeMap::new();
        for name in original {
            *expected.entry(format!("src/sample.rs:{name}")).or_insert(0) += 1;
        }
        let observed = native::observed_with_subject(&root, source, subject);
        assert_eq!(observed, expected, "{source}");
        assert_eq!(
            std::panic::catch_unwind(|| legacy::check_modules(&legacy::modules(source))).is_ok(),
            quantity == 0
        );
    }
}

#[test]
fn frozen_backend_variant_predicate_retains_exact_enum_and_variant_context() {
    let cases = [
        ("", 0),
        ("enum ReactorBackend { Current }", 0),
        ("enum ReactorBackend { Legacy }", 1),
        ("enum ReactorBackend { Legacy(u8) }", 1),
        ("enum ReactorBackend { Legacy { x: u8 } }", 1),
        ("enum Other { Legacy }", 0),
        ("enum r#ReactorBackend { Legacy }", 0),
        ("enum ReactorBackend { r#Legacy }", 0),
        ("mod child { enum ReactorBackend { Legacy } }", 0),
        ("fn f() { enum ReactorBackend { Legacy } }", 0),
        ("#[cfg(any())] enum ReactorBackend { Legacy }", 1),
        ("enum ReactorBackend { #[cfg(any())] Legacy }", 1),
        ("enum ReactorBackend { Legacy, Legacy }", 2),
        (
            "enum ReactorBackend { Legacy } enum ReactorBackend { Legacy }",
            2,
        ),
        (
            "macro_rules! hidden { () => { enum ReactorBackend { Legacy } }; }",
            0,
        ),
        (
            "const TEXT: &str = \"enum ReactorBackend { Legacy }\"; // enum ReactorBackend { Legacy }",
            0,
        ),
        (
            "#[doc={ enum ReactorBackend { Legacy } \"\" }] struct X;",
            0,
        ),
        (
            "enum ReactorBackend { Other = { enum ReactorBackend { Legacy } 0 } }",
            0,
        ),
    ];
    let root = root("variants");
    let subject = "kind='written-file-enum-variants',variants=['ReactorBackend::Legacy']";
    for (source, quantity) in cases {
        assert_eq!(legacy::has_variant(source), quantity > 0, "{source}");
        let observed = native::observed_with_subject(&root, source, subject);
        let expected = if quantity == 0 {
            BTreeMap::new()
        } else {
            BTreeMap::from([("src/sample.rs:ReactorBackend::Legacy".into(), quantity)])
        };
        assert_eq!(observed, expected, "{source}");
        assert_eq!(
            std::panic::catch_unwind(|| legacy::check_variant(legacy::has_variant(source))).is_ok(),
            quantity == 0
        );
    }
}

fn root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .canonicalize()
        .expect("canonical temporary directory")
        .join(format!(
            "zrail-file-{name}-syntax-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
}
