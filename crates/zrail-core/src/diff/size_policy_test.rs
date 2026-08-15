//! Optional source-size enforcement has an explicit semantic permission direction.

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{ChangeKind, compare_architecture, load_contract};

static FIXTURE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn removing_size_enforcement_grants_permission_and_adding_it_revokes() {
    let root = std::env::temp_dir().join(format!(
        "zrail-size-policy-{}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("reset fixture");
    }
    fs::create_dir_all(&root).expect("create fixture");
    fs::write(root.join("rust.toml"), contract(false)).expect("write Rust contract");
    fs::write(root.join("zsumz.toml"), contract(true)).expect("write zsumz contract");
    let rust = load_contract(&root, Path::new("rust.toml"))
        .expect("load contract without size policy")
        .contract;
    let zsumz = load_contract(&root, Path::new("zsumz.toml"))
        .expect("load contract with size policy")
        .contract;

    assert!(rust.source.rust.size.is_none());
    assert!(
        compare_architecture(&zsumz, None, &rust, None)
            .changes
            .iter()
            .any(|change| {
                change.kind == ChangeKind::Grant && change.subject == "source.rust.size"
            })
    );
    assert!(
        compare_architecture(&rust, None, &zsumz, None)
            .changes
            .iter()
            .any(|change| {
                change.kind == ChangeKind::Revoke && change.subject == "source.rust.size"
            })
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

fn contract(with_size: bool) -> String {
    let mut contract = String::from(
        r#"schema = 1
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

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
deny_methods = []
deny_macros = []
"#,
    );
    if with_size {
        contract.push_str(
            r"
[source.rust.size.facade]
target = 300
hard = 300
[source.rust.size.implementation]
target = 300
hard = 300
[source.rust.size.test]
target = 300
hard = 300
[source.rust.size.auxiliary]
target = 300
hard = 300
",
        );
    }
    contract
}
