//! Three old fallible stages and native atomic failure are independently observed.

use super::{
    fault_io::{Fault, Injected},
    injected_oracle,
    model::{self, Fixture, ROOTS},
};
use crate::inventory::{DirectoryIo, scan_repository_using};
use serde::Serialize;
use std::{
    cell::Cell,
    fs,
    io::{self, ErrorKind},
    panic::{AssertUnwindSafe, catch_unwind},
};

#[derive(Serialize)]
pub(super) struct Row {
    case: String,
    legacy_error: String,
    native_error: String,
    native_operations: usize,
}

pub(super) fn run() -> Vec<Row> {
    let mut rows = Vec::new();
    for root in ROOTS {
        for source in [false, true] {
            for kind in [ErrorKind::PermissionDenied, ErrorKind::NotFound] {
                for fault in [
                    Fault::Directory,
                    Fault::EntryFirst,
                    Fault::EntryLast,
                    Fault::Metadata,
                ] {
                    let fixture = Fixture::new();
                    let directory = fixture.root.join(root).join("nested");
                    fs::create_dir(&directory).expect("nested fixture");
                    if source {
                        fs::write(directory.join("visible.rs"), "// source").expect("source");
                    }
                    let injected = Injected {
                        path: directory.clone(),
                        fault,
                        kind,
                        hits: Cell::new(0),
                    };
                    let original = catch_unwind(AssertUnwindSafe(|| match fault {
                        Fault::Directory => injected_oracle::directory(
                            &directory,
                            Err(io::Error::new(
                                kind,
                                "rc9 explicitly injected filesystem failure",
                            )),
                        ),
                        Fault::EntryFirst | Fault::EntryLast => injected_oracle::entries(
                            injected
                                .read_dir(&directory)
                                .expect("real entries and injection"),
                        ),
                        Fault::Metadata => {
                            let entry = fs::read_dir(fixture.root.join(root))
                                .expect("real directory")
                                .map(Result::unwrap)
                                .find(|entry| entry.path() == directory)
                                .expect("real nested entry");
                            injected_oracle::inspect(&injected_oracle::TypeEntry { entry, kind });
                        }
                    }));
                    let legacy_error = model::panic_text(
                        original
                            .expect_err("exact original failure clause")
                            .as_ref(),
                    );
                    let (old_stage, new_stage) = match fault {
                        Fault::Directory => (
                            format!("read directory {}", directory.display()),
                            format!("read {}", directory.display()),
                        ),
                        Fault::EntryFirst | Fault::EntryLast => {
                            ("read directory entry".into(), "read entry".into())
                        }
                        Fault::Metadata => (
                            format!("inspect {}", directory.display()),
                            format!("inspect {}", directory.display()),
                        ),
                    };
                    assert_eq!(
                        legacy_error,
                        format!("{old_stage}: rc9 explicitly injected filesystem failure")
                    );
                    injected.hits.set(0);
                    let native_error = scan_repository_using(&fixture.root, &[], true, &injected)
                        .expect_err("no partial inventory")
                        .to_string();
                    assert_eq!(
                        native_error,
                        format!("{new_stage}: rc9 explicitly injected filesystem failure")
                    );
                    assert_eq!(injected.hits.get(), 1);
                    let replace = |message: String| {
                        message.replace(fixture.root.to_str().expect("root"), "$ROOT")
                    };
                    rows.push(Row {
                        case: format!("{root}:{source}:{kind:?}:{fault:?}"),
                        legacy_error: replace(legacy_error),
                        native_error: replace(native_error),
                        native_operations: injected.hits.get(),
                    });
                }
            }
        }
    }
    assert_eq!(rows.len(), 96);
    rows
}

#[test]
fn traversal_injected_errors_preserve_each_original_failure_stage() {
    run();
}

#[test]
fn traversal_injected_failure_clauses_are_exact_frozen_excerpts() {
    model::source_binding();
    let original = fs::read_to_string(
        model::project().join("crates/zrail-rust/src/rules/size/snapshot/legacy_driver.rs"),
    )
    .expect("frozen oracle");
    let injected = fs::read_to_string(
        model::project().join("crates/zrail-rust/tests/rc9_traversal/injected_oracle.rs"),
    )
    .expect("expression harness");
    for clause in [
        ".unwrap_or_else(|error| panic!(\"read directory {}: {error}\", directory.display()))",
        "        .map(|entry| entry.unwrap_or_else(|error| panic!(\"read directory entry: {error}\")))\n        .collect::<Vec<_>>();",
        ".unwrap_or_else(|error| panic!(\"inspect {}: {error}\", path.display()))",
    ] {
        assert!(
            original.contains(clause) && injected.contains(clause),
            "unchanged clause"
        );
    }
}
