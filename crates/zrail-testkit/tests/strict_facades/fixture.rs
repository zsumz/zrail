//! Tiny dependency-free repositories isolate facade diagnostics from lock failures.

use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct Repository(pub(super) PathBuf);

impl Repository {
    pub(super) fn new(mode: &str, role: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zrail-strict-facades-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset fixture");
        }
        fs::create_dir_all(root.join("src")).expect("create source root");
        fs::create_dir_all(root.join("tests/suite")).expect("create test root");
        let repository = Self(root);
        repository.write(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        );
        repository.write(
            "src/lib.rs",
            "//! Wiring.\nmod child;\npub use child::State;\n",
        );
        repository.write("src/child.rs", "//! Data.\npub struct State;\n");
        repository.write(
            "tests/suite.rs",
            "//! Scenario wiring.\n#[path = \"suite/cases.rs\"] mod cases;\n",
        );
        repository.write(
            "tests/suite/cases.rs",
            "//! Executable scenario.\n#[test] fn works() {}\n",
        );
        repository.write(
            "zrail.toml",
            &format!("{}\n{role}\n", CONTRACT.replace("MODE", mode)),
        );
        repository
    }

    pub(super) fn write(&self, path: &str, source: &str) {
        fs::write(self.0.join(path), source).expect("write fixture input");
    }

    pub(super) fn lock(&self) {
        zrail_rust::build_lock(&self.0, Path::new("zrail.toml"))
            .expect("complete fixture analysis")
            .write(&self.0.join("zrail.lock"))
            .expect("write fixture-only lock");
    }

    pub(super) fn check(&self) -> zrail_rust::CheckResult {
        zrail_rust::check_repository(&self.0, Path::new("zrail.toml"), Path::new("zrail.lock"))
            .expect("analyze fixture")
    }

    pub(super) fn explain(&self, path: &str) -> zrail_rust::PathExplanation {
        zrail_rust::explain_path(&self.0, Path::new("zrail.toml"), Path::new(path))
            .expect("explain fixture")
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test-owned fixture");
    }
}

pub(super) const TEST_FACADE: &str = r#"
[[source.rust.file_roles]]
path = "tests/suite.rs"
role = "test-facade"
mode = "wiring-only"
reason = "Scenario implementations belong in child modules."
"#;

const CONTRACT: &str = r#"schema = 2
adapters = ["rust"]
[repository]
roots = ["src", "tests"]
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[source.rust]
module_docs = "required"
facades = "MODE"
tests = "sibling"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "deny"
[source.rust.size.facade]
target = 80
hard = 100
[source.rust.size.implementation]
target = 240
hard = 240
[source.rust.size.test]
target = 320
hard = 320
[source.rust.size.auxiliary]
target = 300
hard = 300
"#;
