//! Authority-aware diffs reject stale or incompatible lock inputs.

use crate::{ChangeKind, LOCK_SEMANTICS, LockFile, LockedPackage, load_contract};

use super::compare_architecture_checked;

#[test]
fn stale_contract_lock_is_unknown_and_skips_lock_semantics() {
    let fixture = fixture();
    let bundle =
        load_contract(&fixture, std::path::Path::new("zrail.toml")).expect("load fixture contract");
    let mut before = LockFile::new("0".repeat(64));
    before.packages.push(package("core"));
    let mut after = LockFile::new(&bundle.sha256);
    after.packages.push(package("adapter"));

    let report = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        Some(&before),
        &bundle.contract,
        &bundle.sha256,
        Some(&after),
    );

    assert!(report.denies_grants());
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.subject == "before:contract"
    }));
    assert!(
        report
            .changes
            .iter()
            .all(|change| change.rail != "repository.package")
    );
    reset(&fixture);
}

#[test]
fn incompatible_semantics_are_unknown() {
    let fixture = fixture();
    let bundle =
        load_contract(&fixture, std::path::Path::new("zrail.toml")).expect("load fixture contract");
    let mut lock = LockFile::new(&bundle.sha256);
    lock.semantics = 999;

    let report = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        Some(&lock),
        &bundle.contract,
        &bundle.sha256,
        Some(&lock),
    );

    assert_eq!(report.summary.unknown, 2);
    assert!(
        report
            .changes
            .iter()
            .all(|change| change.rail == "lock.authority")
    );
    reset(&fixture);
}

#[test]
fn mixed_old_and_current_semantics_are_unknown_not_resolved_changes() {
    let fixture = fixture();
    let bundle =
        load_contract(&fixture, std::path::Path::new("zrail.toml")).expect("load fixture contract");
    let mut before = LockFile::new(&bundle.sha256);
    before.semantics = LOCK_SEMANTICS - 1;
    let after = LockFile::new(&bundle.sha256);

    let report = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        Some(&before),
        &bundle.contract,
        &bundle.sha256,
        Some(&after),
    );

    assert_eq!(report.summary.unknown, 1);
    assert_eq!(report.summary.grants, 0);
    assert_eq!(report.summary.debt, 0);
    assert!(report.changes.iter().any(|change| {
        change.kind == ChangeKind::Unknown && change.subject == "before:semantics"
    }));
    assert!(
        report
            .changes
            .iter()
            .all(|change| change.rail == "lock.authority")
    );
    reset(&fixture);
}

#[test]
fn producer_change_with_stable_semantics_is_accepted() {
    let fixture = fixture();
    let bundle =
        load_contract(&fixture, std::path::Path::new("zrail.toml")).expect("load fixture contract");
    let before = LockFile::new(&bundle.sha256);
    let mut after = before.clone();
    after.producer = "0.0.2".into();

    let report = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        Some(&before),
        &bundle.contract,
        &bundle.sha256,
        Some(&after),
    );

    assert!(report.changes.is_empty());
    reset(&fixture);
}

#[test]
fn missing_lock_authority_is_unknown_on_either_side() {
    let fixture = fixture();
    let bundle =
        load_contract(&fixture, std::path::Path::new("zrail.toml")).expect("load fixture contract");
    let lock = LockFile::new(&bundle.sha256);

    let missing_before = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        None,
        &bundle.contract,
        &bundle.sha256,
        Some(&lock),
    );
    let missing_after = compare_architecture_checked(
        &bundle.contract,
        &bundle.sha256,
        Some(&lock),
        &bundle.contract,
        &bundle.sha256,
        None,
    );

    for (report, subject) in [
        (missing_before, "before:missing"),
        (missing_after, "after:missing"),
    ] {
        assert!(report.denies_grants());
        assert!(report.changes.iter().any(|change| {
            change.kind == ChangeKind::Unknown
                && change.rail == "lock.authority"
                && change.subject == subject
        }));
    }
    reset(&fixture);
}

fn package(name: &str) -> LockedPackage {
    LockedPackage {
        name: name.into(),
        dependencies: Vec::new(),
    }
}

fn fixture() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-checked-diff-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    reset(&root);
    std::fs::create_dir_all(&root).expect("create fixture");
    std::fs::write(
        root.join("zrail.toml"),
        concat!(
            "schema = 1\nadapters = [\"rust\"]\n\n",
            "[repository]\nroots = [\".\"]\nexclude = []\n",
            "workspace_members = \"exact\"\nnested_git = \"deny\"\n",
            "submodules = \"deny\"\nsymlinks = \"inside\"\n\n",
            "[dependencies]\nmode = \"locked\"\n",
            "unassigned_packages = \"allow\"\ncycles = \"deny\"\n\n",
            "[source.rust]\nmodule_docs = \"allow\"\nfacades = \"allow\"\n",
            "tests = \"allow\"\n\n[source.rust.hygiene]\nunsafe = \"allow\"\n",
            "lint_suppressions = \"allow\"\ndeny_methods = []\ndeny_macros = []\n\n",
            "[source.rust.size.facade]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.implementation]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.test]\ntarget = 300\nhard = 300\n\n",
            "[source.rust.size.auxiliary]\ntarget = 300\nhard = 300\n",
        ),
    )
    .expect("write fixture contract");
    root
}

fn reset(root: &std::path::Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("reset fixture");
    }
}
