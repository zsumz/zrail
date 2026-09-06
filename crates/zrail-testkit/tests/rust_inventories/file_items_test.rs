//! File-level declarations remain distinct from nested code, paths, and physical mounting.

use super::support::{configured_subject, coverage, incomplete, violation};

const DATA: &str = "//! Types.\npub struct State;\n";

#[test]
fn absent_retired_modules_remain_forbidden_and_nested_names_do_not_select() {
    let repository = configured_subject(
        "kind='written-file-modules',names=['tls']",
        "kind='count',maximum=0",
        DATA,
    );
    repository.lock();
    assert_eq!(
        repository.check().report.status,
        zrail_core::ReportStatus::Pass
    );
    repository.write(
        "src/child.rs",
        &format!("{DATA}mod current {{ mod tls {{}} }}\n"),
    );
    assert!(coverage(&repository).rust_inventories[0].satisfied);
    for declaration in ["mod tls {}", "#[cfg(any())] mod tls {}", "pub mod tls {}"] {
        repository.write("src/child.rs", &format!("{DATA}{declaration}\n"));
        violation(&repository);
    }
    repository.write("src/child.rs", &format!("{DATA}mod tls;\n"));
    std::fs::create_dir(repository.0.join("src/child")).expect("external module directory");
    repository.write("src/child/tls.rs", "//! External module.\n");
    violation(&repository);
    let report = coverage(&repository);
    assert_eq!(
        report.rust_inventories[0].claim,
        "authored-rust-file-module-syntax"
    );
    assert_eq!(report.rust_inventories[0].observed_count, 1);
    repository.write("src/child.rs", "mod {");
    incomplete(&repository);
}

#[test]
fn exact_variant_pairs_ignore_unrelated_enums_but_reject_selected_shapes() {
    let subject = "kind='written-file-enum-variants',variants=['Backend::Legacy']";
    let repository = configured_subject(subject, "kind='count',maximum=0", DATA);
    for source in [
        "enum Other { Legacy }",
        "mod nested { enum Backend { Legacy } }",
        "enum Backend { Current }",
        "enum r#Backend { Legacy }",
        "enum Backend { r#Legacy }",
    ] {
        repository.write("src/child.rs", &format!("{DATA}{source}\n"));
        assert!(
            coverage(&repository).rust_inventories[0].satisfied,
            "{source}"
        );
    }
    for source in [
        "enum Backend { Legacy }",
        "enum Backend { Legacy(u8) }",
        "enum Backend { Legacy { value: u8 } }",
        "enum Backend { #[cfg(any())] Legacy }",
    ] {
        repository.write("src/child.rs", &format!("{DATA}{source}\n"));
        violation(&repository);
        let report = coverage(&repository);
        assert_eq!(
            report.rust_inventories[0].claim,
            "authored-rust-file-enum-variant-syntax"
        );
        assert_eq!(report.rust_inventories[0].counts[0].name, "Backend::Legacy");
    }
}

#[test]
fn required_file_items_count_physical_cfg_occurrences_and_fail_on_omission() {
    let source =
        format!("{DATA}enum Backend {{ #[cfg(test)] Legacy, #[cfg(not(test))] Legacy }}\n");
    let repository = configured_subject(
        "kind='written-file-enum-variants',variants=['Backend::Legacy']",
        "kind='exact-counts',counts=[{path='src/child.rs',name='Backend::Legacy',count=2}]",
        &source,
    );
    let report = coverage(&repository);
    assert!(report.rust_inventories[0].satisfied);
    assert_eq!(report.rust_inventories[0].observed_count, 2);
    assert_eq!(
        repository.explain("src/child.rs").rust_inventories,
        report.rust_inventories
    );
    assert_eq!(
        coverage(&repository).json().expect("repeat"),
        report.json().expect("report")
    );
    repository.write("src/child.rs", DATA);
    violation(&repository);
}
