//! Source-classification examples for workspace and root packages.

use super::{FileClass, classify_path};

#[test]
fn classifies_workspace_source_shapes() {
    assert_eq!(classify_path("crates/a/src/lib.rs", &[]), FileClass::Facade);
    assert_eq!(
        classify_path("crates/a/src/worker.rs", &[]),
        FileClass::Implementation
    );
    assert_eq!(
        classify_path("crates/a/src/worker_test.rs", &[]),
        FileClass::Test
    );
}

#[test]
fn classifies_root_package_tests_and_auxiliary_programs() {
    assert_eq!(classify_path("tests/api.rs", &[]), FileClass::Test);
    assert_eq!(classify_path("examples/demo.rs", &[]), FileClass::Auxiliary);
    assert_eq!(classify_path("src/bin/tool.rs", &[]), FileClass::Auxiliary);
    assert_eq!(classify_path("src/main.rs", &[]), FileClass::EntryPoint);
}

#[test]
fn recognizes_only_conventional_test_filenames() {
    for path in [
        "src/tests.rs",
        "src/raft_tests.rs",
        "src/raft_test.rs",
        "src/tests/helpers.rs",
    ] {
        assert_eq!(classify_path(path, &[]), FileClass::Test, "{path}");
    }
    for path in [
        "src/contest.rs",
        "src/latest.rs",
        "src/test.rs",
        "src/tests_support.rs",
    ] {
        assert_eq!(
            classify_path(path, &[]),
            FileClass::Implementation,
            "{path}"
        );
    }
}

#[test]
fn generated_roots_override_ordinary_source_shape() {
    let generated = [zrail_core::GeneratedSourceContract {
        root: "src/generated".into(),
        manifest: "src/generated/MANIFEST.json".into(),
        inputs: vec!["schema/**".into()],
        target: 1_000,
        hard: 2_000,
        reason: "compiler output".into(),
        auxiliary: Vec::new(),
    }];

    assert_eq!(
        classify_path("src/generated/mod.rs", &generated),
        FileClass::Generated
    );
}
