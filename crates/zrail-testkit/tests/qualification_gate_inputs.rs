//! Qualification gate inputs are locked, required, and checked independently of the wrapper.

use std::{
    fs,
    path::{Path, PathBuf},
};

use zrail_rust::{build_lock, check_repository};

#[test]
fn gate_input_bytes_are_part_of_the_candidate_lock() {
    let root = fixture_root();

    let lock = build_lock(&root, Path::new("zrail.toml")).expect("build fixture lock");

    let gate = lock
        .gates
        .iter()
        .find(|gate| gate.name == "check")
        .expect("locked check gate");
    assert_eq!(gate.inputs.len(), 1);
    assert_eq!(gate.inputs[0].path, "scripts/helper");
}

#[test]
fn missing_and_changed_gate_inputs_fail_closed() {
    let root = copy_fixture("gate-input-drift");
    fs::remove_file(root.join("scripts/helper")).expect("remove copied helper");
    let missing = check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check missing gate input");
    assert!(
        missing
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "QUAL-003")
    );

    fs::write(root.join("scripts/helper"), "changed\n").expect("change copied helper");
    let changed = check_repository(&root, Path::new("zrail.toml"), Path::new("zrail.lock"))
        .expect("check changed gate input");
    assert!(
        changed
            .report
            .findings
            .iter()
            .any(|finding| finding.id == "LOCK-026")
    );
    fs::remove_dir_all(root).expect("remove copied fixture");
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evidence_good")
}

fn copy_fixture(name: &str) -> PathBuf {
    let target = std::env::temp_dir().join(format!("zrail-{name}-{}", std::process::id()));
    if target.exists() {
        fs::remove_dir_all(&target).expect("reset copied fixture");
    }
    copy_tree(&fixture_root(), &target);
    target
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create copied fixture directory");
    for entry in fs::read_dir(source).expect("read fixture directory") {
        let entry = entry.expect("read fixture entry");
        let destination = target.join(entry.file_name());
        if entry.file_type().expect("read fixture type").is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).expect("copy fixture file");
        }
    }
}
