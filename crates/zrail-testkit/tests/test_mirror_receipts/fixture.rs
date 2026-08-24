//! Generic temporary Cargo repository for mirror and receipt integration tests.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use zrail_core::{TestExecutionIdentity, TestMirrorContract, test_mirror_input_sha256};
use zrail_rust::{CheckResult, build_lock, check_repository};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

pub(super) struct MirrorFixture {
    root: PathBuf,
}

impl MirrorFixture {
    pub(super) fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "zrail-test-mirror-{label}-{}-{serial}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset mirror fixture");
        }
        for directory in ["src", "tests", "evidence"] {
            fs::create_dir_all(root.join(directory)).expect("create fixture directory");
        }
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='mirror-fixture'\nversion='0.0.0'\nedition='2024'\n",
        )
        .expect("write Cargo manifest");
        fs::write(
            root.join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = \"mirror-fixture\"\nversion = \"0.0.0\"\n",
        )
        .expect("write Cargo lock");
        fs::write(
            root.join("src/lib.rs"),
            "//! Mirror fixture.\npub mod state;\n",
        )
        .expect("write library facade");
        fs::write(
            root.join("src/state.rs"),
            "//! State behavior.\npub fn transition(value: usize) -> usize { value + 1 }\n",
        )
        .expect("write production source");
        fs::write(
            root.join("tests/state_test.rs"),
            concat!(
                "//! State tests.\n",
                "#[test]\nfn state_transitions() {\n",
                "    assert_eq!(mirror_fixture::state::transition(1), 2);\n}\n",
            ),
        )
        .expect("write integration test");
        let fixture = Self { root };
        fixture.write_contract("src/state.rs", "tests/state_test.rs", "state_transitions");
        fixture
    }

    pub(super) fn path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    pub(super) fn write_contract(&self, production: &str, test: &str, name: &str) {
        fs::write(self.path("zrail.toml"), contract(production, test, name))
            .expect("write mirror contract");
    }

    pub(super) fn write_valid_receipt(&self, producer: &str, test: &str, status: &str) {
        self.write_valid_receipt_for(
            "src/state.rs",
            "tests/state_test.rs",
            producer,
            test,
            status,
        );
    }

    pub(super) fn write_valid_receipt_for(
        &self,
        production: &str,
        test_path: &str,
        producer: &str,
        test: &str,
        status: &str,
    ) {
        let digest = input_digest(&self.root, production, test_path, test);
        self.write_receipt(producer, &digest, test, status);
    }

    pub(super) fn write_receipt(
        &self,
        producer: &str,
        input_sha256: &str,
        test: &str,
        status: &str,
    ) {
        fs::write(
            self.path("evidence/state.json"),
            format!(
                concat!(
                    "{{\"schema\":2,\"producer\":\"{}\",",
                    "\"input_sha256\":\"{}\",",
                    "\"execution\":{{\"command\":\"{}\",\"package\":\"mirror-fixture\",",
                    "\"default_features\":true,\"features\":[],\"target\":\"{}\",",
                    "\"toolchain\":\"{}\"}},",
                    "\"tests\":[{{\"id\":\"{}\",\"status\":\"{}\"}}]}}\n",
                ),
                producer, input_sha256, COMMAND, TARGET, TOOLCHAIN, test, status
            ),
        )
        .expect("write execution receipt");
    }

    pub(super) fn write_candidate_lock(&self) -> zrail_core::LockFile {
        let lock = build_lock(&self.root, Path::new("zrail.toml")).expect("build candidate lock");
        lock.write(&self.path("zrail.lock"))
            .expect("write candidate lock");
        lock
    }

    pub(super) fn check(&self) -> CheckResult {
        check_repository(&self.root, Path::new("zrail.toml"), Path::new("zrail.lock"))
            .expect("check mirror fixture")
    }

    pub(super) fn has(checked: &CheckResult, id: &str) -> bool {
        checked
            .report
            .findings
            .iter()
            .any(|finding| finding.id == id)
    }
}

impl Drop for MirrorFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove mirror fixture");
    }
}

fn input_digest(root: &Path, production: &str, test: &str, name: &str) -> String {
    let mirror = mirror_contract(production, test, name);
    let owned = mirror
        .inputs
        .iter()
        .map(|path| {
            (
                path.as_str(),
                fs::read(root.join(path)).expect("read reviewed mirror input"),
            )
        })
        .collect::<Vec<_>>();
    let reviewed = owned
        .iter()
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<Vec<_>>();
    test_mirror_input_sha256(
        &mirror,
        &fs::read(root.join(production)).expect("read production input"),
        &fs::read(root.join(test)).expect("read test input"),
        &reviewed,
    )
}

const COMMAND: &str = concat!(
    "cargo test --package mirror-fixture --test state_test state_transitions ",
    "--target x86_64-unknown-linux-gnu",
);
const TARGET: &str = "x86_64-unknown-linux-gnu";
const TOOLCHAIN: &str = "rustc 1.90.0 (example 2026-01-01)";

fn mirror_contract(production: &str, test: &str, name: &str) -> TestMirrorContract {
    TestMirrorContract {
        production: production.into(),
        test: test.into(),
        name: name.into(),
        receipt: "evidence/state.json".into(),
        inputs: vec![
            "Cargo.lock".into(),
            "Cargo.toml".into(),
            "src/lib.rs".into(),
        ],
        execution: TestExecutionIdentity {
            command: COMMAND.into(),
            package: "mirror-fixture".into(),
            default_features: true,
            features: Vec::new(),
            target: TARGET.into(),
            toolchain: TOOLCHAIN.into(),
        },
        reason: "Exact public-surface behavior".into(),
    }
}

fn contract(production: &str, test: &str, name: &str) -> String {
    format!(
        r#"schema = 2
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

[[source.rust.test_mirrors]]
production = "{production}"
test = "{test}"
name = "{name}"
receipt = "evidence/state.json"
inputs = ["Cargo.lock", "Cargo.toml", "src/lib.rs"]
reason = "The exact test exercises the production behavior through its public surface."

[source.rust.test_mirrors.execution]
command = "{COMMAND}"
package = "mirror-fixture"
default_features = true
features = []
target = "{TARGET}"
toolchain = "{TOOLCHAIN}"

[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"
"#
    )
}
