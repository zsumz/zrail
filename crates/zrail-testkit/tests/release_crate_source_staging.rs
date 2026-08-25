//! Directory-source staging rejects ambiguous or unsafe crate archives.

#![cfg(unix)]

use std::{
    env, fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[test]
fn staging_binds_the_archive_and_every_regular_file() {
    let fixture = Fixture::new("valid");
    let output = fixture.stage(&fixture.archive);
    assert_success(&output);

    let verified = Command::new("python3")
        .args(["-c", VERIFY_CHECKSUMS])
        .arg(&fixture.archive)
        .arg(fixture.source.join("demo-1.2.3"))
        .output()
        .expect("verify checksum manifest");
    assert_success(&verified);
}

#[test]
fn staging_rejects_traversal_links_duplicates_roots_and_wrong_identity() {
    for attack in [
        "traversal",
        "link",
        "duplicate",
        "second-root",
        "wrong-identity",
    ] {
        let fixture = Fixture::new(attack);
        let output = fixture.stage(&fixture.archive);
        assert!(!output.status.success(), "unsafe {attack} archive passed");
        assert!(!fixture.source.join("demo-1.2.3").exists());
        assert!(!fixture.root.join("escape").exists());
    }
}

#[test]
fn staging_rejects_archive_symlinks_and_existing_package_directories() {
    let fixture = Fixture::new("valid");
    let linked = fixture.root.join("linked.crate");
    symlink(&fixture.archive, &linked).expect("create archive symlink");
    assert!(!fixture.stage(&linked).status.success());

    assert_success(&fixture.stage(&fixture.archive));
    assert!(!fixture.stage(&fixture.archive).status.success());
}

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    archive: PathBuf,
}

impl Fixture {
    fn new(mode: &str) -> Self {
        let root = temporary_directory();
        let source = root.join("source");
        let archive = root.join("demo.crate");
        fs::create_dir_all(&source).expect("create fixture source");
        let output = Command::new("python3")
            .args(["-c", CREATE_ARCHIVE])
            .arg(&archive)
            .arg(mode)
            .output()
            .expect("create crate archive");
        assert_success(&output);
        Self {
            root,
            source,
            archive,
        }
    }

    fn stage(&self, archive: &Path) -> Output {
        Command::new("python3")
            .arg(repository_root().join("scripts/stage-crate-source.py"))
            .args(["--archive", archive.to_str().expect("UTF-8 archive path")])
            .args([
                "--directory",
                self.source.to_str().expect("UTF-8 source path"),
            ])
            .args(["--name", "demo", "--version", "1.2.3"])
            .output()
            .expect("run staging helper")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn temporary_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "zrail-crate-source-{}-{nonce}-{sequence}",
        std::process::id()
    ))
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("testkit lives below repository root")
        .to_path_buf()
}

const CREATE_ARCHIVE: &str = r#"
import io, pathlib, sys, tarfile

archive, mode = pathlib.Path(sys.argv[1]), sys.argv[2]
root = "demo-1.2.3"
version = "9.9.9" if mode == "wrong-identity" else "1.2.3"
members = [
    (f"{root}/Cargo.toml", f'[package]\nname = "demo"\nversion = "{version}"\nedition = "2021"\n'),
    (f"{root}/README.md", "readme\n"),
]
if mode == "traversal":
    members.append((f"{root}/../escape", "escaped\n"))
elif mode == "duplicate":
    members.append((f"{root}/README.md", "duplicate\n"))
elif mode == "second-root":
    members.append(("other-1.2.3/file", "other\n"))

with tarfile.open(archive, "w:gz") as bundle:
    for name, text in members:
        data = text.encode()
        member = tarfile.TarInfo(name)
        member.size, member.mode, member.mtime = len(data), 0o644, 0
        bundle.addfile(member, io.BytesIO(data))
    if mode == "link":
        member = tarfile.TarInfo(f"{root}/linked")
        member.type = tarfile.SYMTYPE
        member.linkname = "README.md"
        bundle.addfile(member)
"#;

const VERIFY_CHECKSUMS: &str = r#"
import hashlib, json, pathlib, sys

archive, root = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
checksums = json.loads((root / ".cargo-checksum.json").read_text())
assert checksums["package"] == hashlib.sha256(archive.read_bytes()).hexdigest()
files = {path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
         for path in root.rglob("*") if path.is_file() and path.name != ".cargo-checksum.json"}
assert checksums["files"] == files
"#;
