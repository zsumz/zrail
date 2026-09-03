//! External macro globs bind only through checksum-matched Cargo cache archives.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use flate2::{Compression, write::GzEncoder};
use zrail_core::{AnalysisQuality, ReportStatus};
use zrail_rust::{build_lock, check_repository, explain_path};

const CHILD_MODE: &str = "ZRAIL_EXTERNAL_MACRO_CHILD";
const CHILD_ROOT: &str = "ZRAIL_EXTERNAL_MACRO_ROOT";

#[test]
fn checksum_matched_external_glob_is_exact_and_tampering_is_unknown() {
    let root = fixture_root();
    reset(&root);
    let cargo_home = root.join("cargo-home");
    fs::create_dir_all(root.join("src")).expect("create fixture source");
    fs::create_dir_all(cargo_home.join("registry/cache/test-index"))
        .expect("create fixture Cargo cache");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);

    let archive = crate_archive();
    let checksum = zrail_core::sha256_hex(&archive);
    let archive_path = cargo_home
        .join("registry/cache/test-index")
        .join("external-1.2.3.crate");
    fs::write(&archive_path, &archive).expect("write fixture crate archive");
    write(&root, "Cargo.lock", &cargo_lock(&checksum));

    write(
        &root,
        "src/lib.rs",
        "//! Library.\nuse external::prelude::*;\npub fn run() { reviewed!(); }\n",
    );
    run_child(&root, &cargo_home, "pass");

    write(
        &root,
        "src/lib.rs",
        "//! Library.\nuse external::prelude::*;\npub fn run() { hidden!(); }\n",
    );
    run_child(&root, &cargo_home, "absent");

    let mut tampered = archive;
    tampered.push(0);
    fs::write(&archive_path, tampered).expect("tamper fixture crate archive");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nuse external::prelude::*;\npub fn run() { reviewed!(); }\n",
    );
    run_child(&root, &cargo_home, "checksum");
    reset(&root);
}

#[test]
fn external_macro_archive_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).expect("child fixture root"));
    let lock = build_lock(&root, "zrail.toml".as_ref()).expect("build child lock");
    lock.write(&root.join("zrail.lock"))
        .expect("write child lock");
    let report = check_repository(&root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
        .expect("check child fixture")
        .report;
    match mode.as_str() {
        "pass" => {
            assert_eq!(report.status, ReportStatus::Pass, "{}", report.human());
            let explanation = explain_path(&root, "zrail.toml".as_ref(), "src/lib.rs".as_ref())
                .expect("explain exact external macro");
            let reviewed = explanation
                .macro_invocations
                .iter()
                .find(|invocation| invocation.written == "reviewed")
                .expect("reviewed macro explanation");
            assert_eq!(reviewed.resolution, AnalysisQuality::Exact);
            assert_eq!(
                reviewed.preferred.as_deref(),
                Some("external::prelude::reviewed")
            );
        }
        "absent" => {
            assert_eq!(report.status, ReportStatus::Fail, "{}", report.human());
            assert!(report.findings.iter().any(|finding| {
                finding.id == "RUST-MACRO-001" && finding.message.contains("hidden")
            }));
            assert!(
                report
                    .findings
                    .iter()
                    .all(|finding| finding.id != "RUST-MACRO-006"),
                "{}",
                report.human()
            );
        }
        "checksum" => {
            assert_eq!(report.status, ReportStatus::Fail, "{}", report.human());
            let bindings = report
                .findings
                .iter()
                .filter(|finding| finding.id == "RUST-MACRO-006")
                .collect::<Vec<_>>();
            assert_eq!(bindings.len(), 1, "{}", report.human());
            assert!(bindings[0].message.contains("Cargo.lock checksum"));
        }
        other => panic!("unexpected child mode {other:?}"),
    }
}

fn run_child(root: &Path, cargo_home: &Path, mode: &str) {
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", "external_macro_archive_child", "--nocapture"])
        .env(CHILD_MODE, mode)
        .env(CHILD_ROOT, root)
        .env("CARGO_HOME", cargo_home)
        .output()
        .expect("run isolated external macro test");
    assert!(
        output.status.success(),
        "child failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn crate_archive() -> Vec<u8> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    append(
        &mut archive,
        "external-1.2.3/Cargo.toml",
        "[package]\nname = \"external\"\nversion = \"1.2.3\"\nedition = \"2024\"\n",
    );
    append(
        &mut archive,
        "external-1.2.3/src/lib.rs",
        "#[macro_use]\npub mod sugar;\npub mod prelude;\n",
    );
    append(
        &mut archive,
        "external-1.2.3/src/sugar.rs",
        concat!(
            "#[macro_export]\nmacro_rules! reviewed { () => {} }\n",
            "#[macro_export]\nmacro_rules! hidden { () => {} }\n",
        ),
    );
    append(
        &mut archive,
        "external-1.2.3/src/prelude.rs",
        "pub use crate::reviewed;\npub use crate::ordinary;\n",
    );
    let encoder = archive.into_inner().expect("finish tar archive");
    encoder.finish().expect("finish gzip archive")
}

fn append(archive: &mut tar::Builder<GzEncoder<Vec<u8>>>, path: &str, source: &str) {
    let mut header = tar::Header::new_gnu();
    header.set_size(source.len() as u64);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    archive
        .append_data(&mut header, path, source.as_bytes())
        .expect("append crate archive member");
}

fn fixture_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "zrail-external-cache-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ))
}

fn write(root: &Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

fn reset(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

fn cargo_lock(checksum: &str) -> String {
    format!(
        r#"version = 4

[[package]]
name = "external"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "{checksum}"

[[package]]
name = "fixture"
version = "0.0.0"
dependencies = ["external"]
"#
    )
}

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.0.0"
edition = "2024"
[dependencies]
external = "1"
"#;

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[[dependencies.crate_root]]
package = "external"
root = "external"
reason = "The published library exposes this crate root."
[dependencies.crate_root.source]
kind = "registry"
requirement = "1"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "deny-unreviewed"
[[source.rust.macros.allow]]
name = "external::prelude::reviewed"
resolution = "exact"
reason = "The checksum-matched prelude explicitly exports this macro."
[source.rust.macros.allow.source]
kind = "cargo-lock"
package = "external"
version = "1.2.3"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#;
