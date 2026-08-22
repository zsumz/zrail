//! Checked-in release helpers create deterministic archives and reviewed notes.

#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn release_archives_are_deterministic_with_exact_members() {
    let root = repository_root();
    let temporary = temporary_directory("archives");
    fs::create_dir_all(&temporary).expect("create release fixture");
    let license = temporary.join("LICENSE");
    let readme = temporary.join("README.md");
    fs::write(&license, "license\n").expect("write license");
    fs::write(&readme, "readme\n").expect("write readme");

    for (suffix, executable) in [("tar.gz", "zrail"), ("zip", "zrail.exe")] {
        let binary = temporary.join(executable);
        let output = temporary.join(format!("zrail-1.2.3-target.{suffix}"));
        fs::write(&binary, b"binary\0bytes").expect("write binary");
        package(&root, &binary, &license, &readme, &output);
        let first = fs::read(&output).expect("read first archive");

        fs::write(&binary, b"binary\0bytes").expect("rewrite binary");
        fs::write(&license, "license\n").expect("rewrite license");
        package(&root, &binary, &license, &readme, &output);
        let second = fs::read(&output).expect("read second archive");

        assert_eq!(
            first, second,
            "{suffix} output depends on filesystem metadata"
        );
        assert_eq!(
            archive_members(&output),
            vec![executable.to_owned(), "LICENSE".into(), "README.md".into()]
        );
    }
    fs::remove_dir_all(temporary).expect("remove release fixture");
}

#[test]
fn release_notes_are_extracted_from_one_exact_version_section() {
    let root = repository_root();
    let temporary = temporary_directory("notes");
    fs::create_dir_all(&temporary).expect("create notes fixture");
    let changelog = temporary.join("CHANGELOG.md");
    let output = temporary.join("notes.md");
    fs::write(
        &changelog,
        "# Changelog\n\n## [1.2.3] - today\n\nReviewed notes.\n\n## [1.2.2]\n\nOld.\n",
    )
    .expect("write changelog");

    let status = Command::new("python3")
        .arg(root.join("scripts/release-notes.py"))
        .args(["1.2.3"])
        .arg(&changelog)
        .arg(&output)
        .status()
        .expect("run release notes helper");
    assert!(status.success());
    assert_eq!(
        fs::read_to_string(&output).expect("read notes"),
        "Reviewed notes.\n"
    );

    let missing = Command::new("python3")
        .arg(root.join("scripts/release-notes.py"))
        .args(["9.9.9"])
        .arg(&changelog)
        .arg(&output)
        .output()
        .expect("run missing notes helper");
    assert!(!missing.status.success());
    fs::remove_dir_all(temporary).expect("remove notes fixture");
}

fn package(root: &Path, binary: &Path, license: &Path, readme: &Path, output: &Path) {
    let status = Command::new("python3")
        .arg(root.join("scripts/package-release.py"))
        .arg("--binary")
        .arg(binary)
        .arg("--license")
        .arg(license)
        .arg("--readme")
        .arg(readme)
        .arg("--output")
        .arg(output)
        .status()
        .expect("run release packager");
    assert!(status.success());
}

fn archive_members(archive: &Path) -> Vec<String> {
    let script = concat!(
        "import sys,tarfile,zipfile; p=sys.argv[1]; ",
        "names=tarfile.open(p,'r:gz').getnames() if p.endswith('.tar.gz') ",
        "else zipfile.ZipFile(p).namelist(); print('\\n'.join(names))"
    );
    let output = Command::new("python3")
        .args(["-c", script])
        .arg(archive)
        .output()
        .expect("inspect archive");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("UTF-8 archive listing")
        .lines()
        .map(str::to_owned)
        .collect()
}

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zrail-release-{label}-{}-{nonce}",
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
