//! Disposable repositories exercise prelude policy through the public API.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Finding, Report};
use zrail_rust::{build_lock, check_repository};

pub(super) struct Repository {
    root: PathBuf,
}

impl Repository {
    pub(super) fn new(name: &str, edition: &str, source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zrail-prelude-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset prelude fixture");
        }
        fs::create_dir_all(root.join("src")).expect("create prelude fixture");
        fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"prelude-fixture\"\nversion = \"0.0.0\"\nedition = \"{edition}\"\n"
            ),
        )
        .expect("write prelude manifest");
        fs::write(root.join("zrail.toml"), CONTRACT).expect("write prelude contract");
        fs::write(root.join("src/lib.rs"), source).expect("write prelude source");
        fs::write(root.join("src/owner.rs"), OWNER).expect("write prelude owner");
        Self { root }
    }

    pub(super) fn write(&self, path: &str, contents: &str) {
        fs::write(self.root.join(path), contents).expect("write prelude fixture file");
    }

    pub(super) fn check(&self) -> Report {
        match build_lock(&self.root, "zrail.toml".as_ref()) {
            Ok(lock) => lock
                .write(&self.root.join("zrail.lock"))
                .expect("write prelude lock"),
            Err(error) if error.to_string().contains("incomplete analysis") => {}
            Err(error) => panic!("build prelude lock: {error}"),
        }
        check_repository(&self.root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
            .expect("check prelude fixture")
            .report
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("remove prelude fixture");
        }
    }
}

pub(super) fn exact<'a>(report: &'a Report, id: &str, rule: &str, path: &str) -> &'a Finding {
    report
        .findings
        .iter()
        .find(|finding| {
            finding.id == id
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Exact
        })
        .unwrap_or_else(|| panic!("missing exact {id}/{rule} in {path}:\n{}", report.human()))
}

pub(super) fn count(report: &Report, id: &str, rule: &str, path: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == id && finding.rule == rule && finding.path.as_deref() == Some(path)
        })
        .count()
}

pub(super) fn assert_no_owner(report: &Report, rule: &str, path: &str) {
    assert!(
        !report.findings.iter().any(|finding| {
            finding.id.starts_with("OWN-")
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
        }),
        "{}",
        report.human()
    );
}

pub(super) fn findings<'a>(
    report: &'a Report,
    id: &str,
    rule: &str,
    path: &str,
) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == id && finding.rule == rule && finding.path.as_deref() == Some(path)
        })
        .collect()
}

const OWNER: &str = r"//! Canonical prelude policy owner.
pub fn own() {
    core::mem::drop(0_u8);
    let _ = std::vec::Vec::<u8>::new();
    let _: core::option::Option<u8> = core::option::Option::Some(1);
    let _: core::option::Option<u8> = core::option::Option::None;
    let _: core::result::Result<u8, u8> = core::result::Result::Ok(1);
    let _: core::result::Result<u8, u8> = core::result::Result::Err(1);
}
";

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
[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "drop-call"
kind = "call"
within = ["src/**"]
match = "core::mem::drop"
allow = ["src/owner.rs"]
reason = "Dropping by policy stays centralized."

[[owner]]
name = "drop-capability"
kind = "capability"
within = ["src/**"]
match = "core::mem::drop"
allow = ["src/owner.rs"]
reason = "The drop capability stays centralized."

[[owner]]
name = "vec-new"
kind = "call"
within = ["src/**"]
match = "std::vec::Vec::new"
allow = ["src/owner.rs"]
reason = "Vector allocation stays centralized."

[[owner]]
name = "option-construction"
kind = "type-construction"
within = ["src/**"]
match = "core::option::Option"
allow = ["src/owner.rs"]
reason = "Option construction stays centralized."

[[owner]]
name = "result-construction"
kind = "type-construction"
within = ["src/**"]
match = "core::result::Result"
allow = ["src/owner.rs"]
reason = "Result construction stays centralized."

[[scope]]
name = "vec-symbols"
include = ["src/lib.rs"]
reason = "The public fixture denies vector symbols."
[scope.symbols]
deny = ["std::vec::Vec"]
"#;
