//! Unix fixtures never follow a cycle, read a FIFO, or leave unreadable directories behind.

use super::{Fixture, NAMES, check, model};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};

#[test]
fn retired_contained_file_links_preserve_extension_selection() {
    for name in NAMES {
        for (file, accepted) in [("old.rs", false), ("old.txt", true), (".rs", true)] {
            let fixture = Fixture::new();
            fixture.write("targets/source", "// target");
            let tree = fixture.root.join("src/reactor").join(name);
            fs::create_dir_all(&tree).expect("tree");
            symlink("../../../targets/source", tree.join(file)).expect("contained file link");
            check(
                &fixture,
                name,
                Some(accepted),
                if accepted { "" } else { "REP-FILE-001" },
            );
        }
    }
}

#[test]
fn retired_directory_links_never_claim_complete_descendant_selection() {
    for name in NAMES {
        for escaping in [false, true] {
            for source in [false, true] {
                let fixture = Fixture::new();
                let target = if escaping {
                    fixture.root.with_extension("external")
                } else {
                    fixture.root.join("targets")
                };
                assert!(!target.exists(), "fresh target owned by this test");
                fs::create_dir(&target).expect("link target");
                let cleanup = ExternalTarget(escaping.then_some(target.clone()));
                if source {
                    fs::write(target.join("old.rs"), "// source").expect("source");
                }
                let tree = fixture.root.join("src/reactor").join(name);
                fs::create_dir_all(&tree).expect("tree");
                symlink(&target, tree.join("linked")).expect("directory link");
                check(&fixture, name, Some(!source), "REP-FILE-006");
                drop(cleanup);
            }
        }
    }
}

#[test]
fn retired_broken_links_are_incomplete_even_when_the_legacy_extension_passes() {
    for name in NAMES {
        for file in ["old.rs", "old.txt"] {
            let fixture = Fixture::new();
            let tree = fixture.root.join("src/reactor").join(name);
            fs::create_dir_all(&tree).expect("tree");
            symlink("missing", tree.join(file)).expect("broken link");
            check(&fixture, name, Some(file != "old.rs"), "REP-FILE-006");
        }
    }
}

#[test]
fn retired_directory_cycles_fail_closed_without_running_the_unbounded_oracle() {
    for name in NAMES {
        let fixture = Fixture::new();
        let tree = fixture.root.join("src/reactor").join(name);
        fs::create_dir_all(&tree).expect("tree");
        symlink(".", tree.join("cycle")).expect("cycle");
        check(&fixture, name, None, "REP-FILE-006");
    }
}

#[test]
fn retired_special_entries_need_non_directory_selection_without_content_reads() {
    for name in NAMES {
        for file in ["old.rs", "old.txt", ".rs"] {
            let fixture = Fixture::new();
            let tree = fixture.root.join("src/reactor").join(name);
            fs::create_dir_all(&tree).expect("tree");
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(tree.join(file))
                    .status()
                    .expect("create real FIFO")
                    .success()
            );
            let old = crate::repository_files::analyze(
                &fixture.root,
                &model::policies().repository.files,
            )
            .expect("original regular-file selection");
            assert!(
                old.findings.is_empty(),
                "the earlier proposal missed special entries"
            );
            check(
                &fixture,
                name,
                Some(file != "old.rs"),
                if file == "old.rs" { "REP-FILE-001" } else { "" },
            );
        }
    }
}

#[test]
#[ignore = "requires a Unix user whose directory permissions are enforced; run explicitly"]
fn retired_unreadable_directories_fail_closed_and_unreadable_files_still_count() {
    for name in NAMES {
        for case in ["empty-directory", "source-directory", "source-file"] {
            let fixture = Fixture::new();
            let tree = fixture.root.join("src/reactor").join(name);
            fs::create_dir_all(&tree).expect("tree");
            let file = tree.join("old.rs");
            if case != "empty-directory" {
                fs::write(&file, "// source").expect("source");
            }
            let path = if case == "source-file" { file } else { tree };
            let restore = Permissions {
                original: fs::metadata(&path).expect("metadata").permissions(),
                path,
            };
            fs::set_permissions(&restore.path, fs::Permissions::from_mode(0o000))
                .expect("deny access");
            if case == "source-file" {
                assert_eq!(
                    fs::read(&restore.path)
                        .expect_err("enforced file permissions")
                        .kind(),
                    std::io::ErrorKind::PermissionDenied
                );
                check(&fixture, name, Some(false), "REP-FILE-001");
            } else {
                assert_eq!(
                    fs::read_dir(&restore.path)
                        .expect_err("enforced directory permissions")
                        .kind(),
                    std::io::ErrorKind::PermissionDenied
                );
                check(&fixture, name, Some(true), "REP-FILE-006");
            }
            drop(restore);
        }
    }
}

struct Permissions {
    path: PathBuf,
    original: fs::Permissions,
}
impl Drop for Permissions {
    fn drop(&mut self) {
        fs::set_permissions(&self.path, self.original.clone())
            .expect("restore test-owned permissions");
    }
}

struct ExternalTarget(Option<PathBuf>);
impl Drop for ExternalTarget {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            fs::remove_dir_all(path).expect("remove only test-owned target");
        }
    }
}
