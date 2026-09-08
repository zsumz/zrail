//! Injected iterator/inspection errors never become trusted partial inventory or false absence.

use super::{
    entry_oracle,
    fault_io::{Fault, Injected},
    model::{self, Fixture, NAMES},
};
use crate::inventory::{DirectoryIo, scan_repository_using};
use std::{cell::Cell, fs, io::ErrorKind};

#[test]
fn retired_entry_expression_is_an_exact_frozen_excerpt() {
    let source = fs::read_to_string(
        model::project()
            .join("crates/zrail-testkit/tests/fixtures/rc9/declarations/release_graph.rs"),
    )
    .expect("frozen source");
    assert_eq!(
        zrail_core::sha256_hex(source.as_bytes()),
        "e30e48c6f261690844c9db7ef14d02316740c4b4d8eec322241268d454c947ff"
    );
    let excerpt = source
        .lines()
        .skip(51)
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");
    let harness = fs::read_to_string(
        model::project().join("crates/zrail-rust/tests/rc9_declarations/retired_entry_oracle.rs"),
    )
    .expect("expression oracle");
    assert!(
        harness.contains(&excerpt),
        "unchanged original lines 52..59, including filter_map(Result::ok)"
    );
}

#[test]
fn retired_injected_entries_are_ignored_by_the_original_expression_but_rejected_by_inventory() {
    let mut cases = 0;
    for name in NAMES {
        for source in [false, true] {
            for fault in [Fault::EntryFirst, Fault::EntryLast] {
                for kind in [ErrorKind::PermissionDenied, ErrorKind::NotFound] {
                    let fixture = Fixture::new();
                    let path = fixture.root.join("src/reactor").join(name);
                    fs::create_dir_all(&path).expect("tree");
                    if source {
                        fs::write(path.join("old.rs"), "// source").expect("source");
                    }
                    let injected = Injected {
                        path,
                        fault,
                        kind,
                        hits: Cell::new(0),
                    };
                    assert_eq!(
                        entry_oracle::contains_from_entries(
                            injected
                                .read_dir(&injected.path)
                                .expect("real directory plus injected entry")
                        ),
                        source
                    );
                    assert_eq!(
                        injected.hits.replace(0),
                        1,
                        "exactly one legacy expression injection"
                    );
                    let error = scan_repository_using(&fixture.root, &[], true, &injected)
                        .expect_err("never return partial inventory")
                        .to_string();
                    assert_eq!(
                        error,
                        "read entry: rc9 explicitly injected filesystem failure"
                    );
                    assert_eq!(
                        injected.hits.get(),
                        1,
                        "the same injected directory was visited"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 56);
}

#[test]
fn retired_injected_directory_and_metadata_failures_abort_the_entire_scan() {
    let mut cases = 0;
    for name in NAMES {
        for fault in [Fault::Directory, Fault::Metadata] {
            for kind in [ErrorKind::PermissionDenied, ErrorKind::NotFound] {
                let fixture = Fixture::new();
                let directory = fixture.root.join("src/reactor").join(name);
                fs::create_dir_all(&directory).expect("tree");
                let file = directory.join("old.rs");
                fs::write(&file, "// source").expect("source");
                let path = if matches!(fault, Fault::Directory) {
                    directory
                } else {
                    file
                };
                let injected = Injected {
                    path,
                    fault,
                    kind,
                    hits: Cell::new(0),
                };
                let error = scan_repository_using(&fixture.root, &[], true, &injected)
                    .expect_err("no incomplete inventory")
                    .to_string();
                let stage = if matches!(fault, Fault::Directory) {
                    "read"
                } else {
                    "inspect"
                };
                assert_eq!(
                    error,
                    format!(
                        "{stage} {}: rc9 explicitly injected filesystem failure",
                        injected.path.display()
                    )
                );
                assert_eq!(injected.hits.get(), 1);
                assert!(
                    injected.path.exists(),
                    "no racy deletion or real permission change"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 28);
}
