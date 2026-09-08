//! Real Unix boundaries never read FIFO streams or leave unreadable fixtures behind.

use super::model::{Fixture, ROOTS, Row, observe};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};

pub(super) fn links_and_specials() -> Vec<Row> {
    let mut rows = Vec::new();
    for selected in ROOTS {
        for case in [
            "file-link",
            "directory-link",
            "directory-rs-link",
            "escaping-file-link",
            "broken-link",
            "cycle",
            "root-empty-link",
            "root-source-link",
        ] {
            let fixture = Fixture::new();
            let path = fixture.root.join(selected);
            let target = fixture.root.join("targets");
            fs::create_dir(&target).expect("contained target");
            fs::write(target.join("source"), "// target").expect("file target");
            let outside = fixture.root.with_extension("outside");
            assert!(!outside.exists(), "fresh external target");
            let cleanup = External((case == "escaping-file-link").then_some(outside.clone()));
            match case {
                "file-link" => symlink(target.join("source"), path.join("link.rs")),
                "directory-link" => symlink(&target, path.join("linked")),
                "directory-rs-link" => symlink(&target, path.join("linked.rs")),
                "escaping-file-link" => {
                    fs::create_dir(&outside).expect("external fixture");
                    fs::write(outside.join("source"), "// target").expect("external file");
                    symlink(outside.join("source"), path.join("link.rs"))
                }
                "broken-link" => symlink("absent", path.join("link.rs")),
                "cycle" => symlink(".", path.join("loop.rs")),
                _ => {
                    fs::remove_dir(&path).expect("replace only empty selected fixture root");
                    if case == "root-source-link" {
                        fs::write(target.join("visible.rs"), "// source").expect("root source");
                    }
                    symlink(&target, &path)
                }
            }
            .expect("physical link");
            rows.push(observe(
                &fixture,
                &format!("{selected}:{case}"),
                None,
                if case == "file-link" {
                    ""
                } else {
                    "REP-FILE-006"
                },
            ));
            drop(cleanup);
        }
        for name in ["pipe.rs", "pipe.RS", ".rs"] {
            let fixture = Fixture::new();
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(fixture.root.join(selected).join(name))
                    .status()
                    .expect("real FIFO")
                    .success()
            );
            rows.push(observe(
                &fixture,
                &format!("{selected}:fifo-{name}"),
                None,
                "",
            ));
            let row = rows.last().expect("FIFO row");
            assert!(
                row.observations
                    .iter()
                    .flat_map(|rule| &rule.entries)
                    .filter(|entry| entry.kind == "other")
                    .all(|entry| entry.sha256.is_none() && entry.bytes.is_none())
            );
        }
    }
    assert_eq!(rows.len(), 66);
    rows
}

#[test]
fn traversal_links_and_special_entries_preserve_explicit_boundaries() {
    links_and_specials();
}

pub(super) fn permissions() -> Vec<Row> {
    let mut rows = Vec::new();
    for selected in ROOTS {
        for case in ["unread-empty", "unread-nested", "unread-file"] {
            let fixture = Fixture::new();
            let root = fixture.root.join(selected);
            let path = match case {
                "unread-nested" => {
                    let path = root.join("nested");
                    fs::create_dir(&path).expect("nested empty");
                    path
                }
                "unread-file" => {
                    let path = root.join("source.rs");
                    fs::write(&path, "// unread").expect("source");
                    path
                }
                _ => root,
            };
            let restore = Permissions {
                original: fs::metadata(&path).expect("mode").permissions(),
                path,
            };
            fs::set_permissions(&restore.path, fs::Permissions::from_mode(0o000))
                .expect("deny access");
            let file = case == "unread-file";
            let kind = if file {
                fs::read(&restore.path)
                    .expect_err("enforced file permission")
                    .kind()
            } else {
                fs::read_dir(&restore.path)
                    .expect_err("enforced directory permission")
                    .kind()
            };
            assert_eq!(kind, std::io::ErrorKind::PermissionDenied);
            rows.push(observe(
                &fixture,
                &format!("{selected}:{case}"),
                (!file).then_some("read directory "),
                if file { "" } else { "REP-FILE-006" },
            ));
        }
    }
    let fixture = Fixture::new();
    let path = fixture.root.join("outside");
    fs::create_dir(&path).expect("unselected fixture directory");
    let restore = Permissions {
        original: fs::metadata(&path).expect("mode").permissions(),
        path,
    };
    fs::set_permissions(&restore.path, fs::Permissions::from_mode(0o000)).expect("deny access");
    assert_eq!(
        fs::read_dir(&restore.path)
            .expect_err("enforced directory permission")
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    rows.push(observe(
        &fixture,
        "outside:unread-directory",
        None,
        "REP-FILE-006",
    ));
    assert_eq!(rows.len(), 19);
    rows
}

#[test]
#[ignore = "requires Unix permission enforcement, explicitly checked before qualification"]
fn traversal_unread_directories_fail_and_unread_files_need_no_content_read() {
    permissions();
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
struct External(Option<PathBuf>);
impl Drop for External {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            fs::remove_dir_all(path).expect("remove test-owned external target");
        }
    }
}
