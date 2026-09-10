//! Frozen source-root traversal keeps all attempted I/O failures explicit.

use crate::inventory::test_faults as fault_io;
#[path = "injected_test.rs"]
mod injected;
#[path = "injected_oracle.rs"]
mod injected_oracle;
#[path = "inspection_test.rs"]
mod inspection;
#[path = "model.rs"]
mod model;
#[path = "qualification.rs"]
mod qualification;
#[cfg(unix)]
#[path = "unix_test.rs"]
mod unix;

use model::{Fixture, ROOTS, Row, observe};
use std::fs;

#[test]
fn traversal_error_root_comparison_accepts_both_platform_separators() {
    let case = "crates/kafka-driver-core/src:missing-root";
    for error in [
        "read directory $ROOT/crates/kafka-driver-core/src: missing",
        "read directory $ROOT\\crates\\kafka-driver-core\\src: missing",
        "REP-FILE-006: C:\\fixture\\crates\\kafka-driver-core\\src\\.git: incomplete",
    ] {
        assert!(model::error_names_root(error, case));
    }
    assert!(!model::error_names_root(
        "read directory $ROOT/crates/kafka-driver-probe/src: missing",
        case
    ));
}

#[test]
fn traversal_original_function_is_frozen() {
    model::source_binding();
}

fn portable() -> Vec<Row> {
    portable_with(observe)
}

fn portable_with(observe: model::Observer) -> Vec<Row> {
    let mut rows = Vec::new();
    for selected in ROOTS {
        for case in [
            "missing-root",
            "file-root",
            "empty",
            "nested-empty",
            "ordered-source",
            "component-order",
            "git-empty",
            "git-source",
        ] {
            let fixture = Fixture::new();
            let path = fixture.root.join(selected);
            match case {
                "missing-root" | "file-root" => {
                    fs::remove_dir(&path).expect("remove only empty fixture root");
                    if case == "file-root" {
                        fs::write(&path, "not a directory").expect("root file");
                    }
                    let other = ROOTS
                        .iter()
                        .find(|root| **root != selected)
                        .expect("another root");
                    fixture.write(
                        &format!("{other}/surviving.rs"),
                        "// must not mask a missing root",
                    );
                }
                "nested-empty" => {
                    fs::create_dir_all(path.join("nested/empty.rs")).expect("empty directory");
                }
                "ordered-source" => {
                    for name in [
                        "z.rs",
                        "a.rs",
                        "nested/cfg.rs",
                        "UPPER.RS",
                        ".rs",
                        "not-rust.txt",
                    ] {
                        fixture.write(
                            &format!("{selected}/{name}"),
                            "#[cfg(any())] fn hidden() {}\n",
                        );
                    }
                    fixture.write("outside/decoy.rs", "// not selected");
                }
                "component-order" => {
                    fixture.write(&format!("{selected}/a.rs"), "// sibling");
                    fixture.write(&format!("{selected}/a/nested.rs"), "// descendant");
                }
                "git-empty" | "git-source" => {
                    fs::create_dir(path.join(".git")).expect("pruned tree");
                    if case == "git-source" {
                        fs::write(path.join(".git/hidden.rs"), "// source").expect("source");
                    }
                }
                _ => {}
            }
            let missing = matches!(case, "missing-root" | "file-root");
            rows.push(observe(
                &fixture,
                &format!("{selected}:{case}"),
                missing.then_some("read directory "),
                if missing {
                    "REP-FILE-002"
                } else if case.starts_with("git-") {
                    "REP-FILE-006"
                } else {
                    ""
                },
            ));
            if case == "component-order" {
                let row = rows.last().expect("ordering observation");
                assert_eq!(
                    row.legacy_paths,
                    [
                        format!("{selected}/a/nested.rs"),
                        format!("{selected}/a.rs")
                    ]
                );
                assert_eq!(
                    row.native_paths,
                    [
                        format!("{selected}/a.rs"),
                        format!("{selected}/a/nested.rs")
                    ]
                );
            }
        }
    }
    assert_eq!(rows.len(), 48);
    rows
}

#[test]
fn traversal_roots_empty_trees_and_selection_are_explicit() {
    portable();
}

#[test]
#[ignore = "requires a clean committed producer, frozen snapshots and enforced Unix permissions"]
fn qualify_frozen_source_traversal() {
    qualification::run();
}
