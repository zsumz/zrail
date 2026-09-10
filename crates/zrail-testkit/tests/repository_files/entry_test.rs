//! Non-directory assertions bind physical kinds and never open special-file content streams.

use super::support::{configured, coverage, violation};
use std::fs;
use zrail_core::{ReportStatus, RepositoryEntryMode};

const RULE: &str = r#"
[[repository.files]]
name = "retired"
include = ["assets/**/*.rs"]
entry = "non-directory"
reason = "Every non-directory Rust-extension entry is forbidden."
predicate = { kind = "count", minimum = 0, maximum = 0 }
"#;

#[test]
fn non_directory_guards_ignore_directories_but_still_forbid_nested_files() {
    let repository = configured(RULE);
    fs::create_dir_all(repository.0.join("assets/empty.rs")).expect("directory with Rust suffix");
    repository.lock();
    assert_eq!(repository.check().report.status, ReportStatus::Pass);
    assert!(coverage(&repository).repository_files[0].entries.is_empty());
    repository.write("assets/empty.rs/old.rs", "// written source");
    violation(&repository, "retired", "REP-FILE-001");
    let report = coverage(&repository);
    assert_eq!(
        report.repository_files[0].policy.entry,
        RepositoryEntryMode::NonDirectory
    );
    assert_eq!(
        report.repository_files[0].entries[0].path,
        "assets/empty.rs/old.rs"
    );
    assert_eq!(report.repository_files[0].entries[0].kind, "file");
}

#[cfg(unix)]
#[test]
fn non_directory_fifo_observations_are_exposed_and_lock_bound_without_reading_bytes() {
    for selector in ["assets/**/*.rs", "assets/old.rs"] {
        let repository = configured(
            &RULE
                .replace("assets/**/*.rs", selector)
                .replace("maximum = 0", "maximum = 1"),
        );
        fs::create_dir(repository.0.join("assets")).expect("assets");
        let path = repository.0.join("assets/old.rs");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .expect("real FIFO")
                .success()
        );
        repository.lock();
        assert_eq!(repository.check().report.status, ReportStatus::Pass);
        let report = coverage(&repository);
        let entry = &report.repository_files[0].entries[0];
        assert_eq!(entry.kind, "other");
        assert_eq!(entry.sha256, None);
        assert_eq!(entry.bytes, None);
        fs::remove_file(&path).expect("remove only test-owned FIFO");
        repository.write("assets/old.rs", "// now a regular file");
        let result = repository.check();
        assert!(
            result
                .report
                .findings
                .iter()
                .any(|finding| finding.id == "LOCK-028")
        );
        assert_eq!(
            coverage(&repository).repository_files[0].entries[0].kind,
            "file"
        );
    }
}

#[cfg(unix)]
#[test]
fn non_directory_exact_links_classify_contained_targets_and_reject_broken_targets() {
    use std::os::unix::fs::symlink;
    let repository = configured(&RULE.replace("assets/**/*.rs", "assets/link.rs"));
    fs::create_dir(repository.0.join("assets")).expect("assets");
    fs::create_dir(repository.0.join("assets/directory")).expect("directory");
    let link = repository.0.join("assets/link.rs");
    symlink("directory", &link).expect("contained directory link");
    assert!(coverage(&repository).repository_files[0].entries.is_empty());
    fs::remove_file(&link).expect("remove directory link");
    repository.write("assets/target", "// regular target");
    symlink("target", &link).expect("contained file link");
    violation(&repository, "retired", "REP-FILE-001");
    let report = coverage(&repository);
    assert_eq!(report.repository_files[0].entries[0].kind, "symlink");
    assert_eq!(
        report.repository_files[0].entries[0]
            .resolved_path
            .as_deref(),
        Some("assets/target")
    );
    fs::remove_file(repository.0.join("assets/target")).expect("break only test-owned link");
    for result in [
        zrail_rust::build_lock(&repository.0, "zrail.toml".as_ref()).map(|_| ()),
        zrail_rust::governed_surface_report(&repository.0, "zrail.toml".as_ref()).map(|_| ()),
        zrail_rust::check_repository(&repository.0, "zrail.toml".as_ref(), "zrail.lock".as_ref())
            .map(|_| ()),
    ] {
        assert!(
            result
                .expect_err("no partial evidence")
                .to_string()
                .contains("REP-FILE-006")
        );
    }
}
