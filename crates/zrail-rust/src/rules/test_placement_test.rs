//! Test module reachability follows Rust's real directory ownership rules.

use std::collections::BTreeSet;

use crate::source::{ModuleDeclaration, SubmoduleBase};

use super::resolved_module_target;

#[test]
fn parent_module_declarations_reach_sibling_tests() {
    let files = BTreeSet::from(["src/worker.rs", "src/worker_test.rs"]);
    let declaration = declaration("worker_test", None);

    assert_eq!(
        resolved_module_target(
            "src/lib.rs",
            SubmoduleBase::SourceParent,
            &declaration,
            &files,
        )
        .as_deref(),
        Some("src/worker_test.rs")
    );
    assert_eq!(
        resolved_module_target(
            "src/worker.rs",
            SubmoduleBase::FileStemDirectory,
            &declaration,
            &files,
        ),
        None
    );
}

#[test]
fn exact_path_can_link_a_test_from_its_implementation_file() {
    let files = BTreeSet::from(["src/worker.rs", "src/worker_test.rs"]);
    let declaration = declaration("tests", Some("worker_test.rs"));

    assert_eq!(
        resolved_module_target(
            "src/worker.rs",
            SubmoduleBase::FileStemDirectory,
            &declaration,
            &files,
        )
        .as_deref(),
        Some("src/worker_test.rs")
    );
}

fn declaration(name: &str, path: Option<&str>) -> ModuleDeclaration {
    ModuleDeclaration {
        name: name.into(),
        path: path.map(str::to_owned),
        cfg_test: true,
        unresolved_path: false,
        inline_ancestors: Vec::new(),
        span: None,
    }
}
