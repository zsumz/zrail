//! Disposable repositories exercise generic namespace truth through public policy.

use std::{fs, path::PathBuf};

use zrail_core::{AnalysisQuality, Report};
use zrail_rust::{build_lock, check_repository};

pub(super) struct Repository {
    root: PathBuf,
}

impl Repository {
    pub(super) fn new(name: &str, source: &str, owner_source: &str, owner_contract: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zrail-generic-shadow-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset generic-shadow fixture");
        }
        fs::create_dir_all(root.join("src")).expect("create generic-shadow fixture");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"generic-shadow-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("write generic-shadow manifest");
        fs::write(
            root.join("zrail.toml"),
            format!("{BASE_CONTRACT}\n{owner_contract}"),
        )
        .expect("write generic-shadow contract");
        fs::write(root.join("src/lib.rs"), source).expect("write generic-shadow source");
        fs::write(root.join("src/owner.rs"), owner_source).expect("write generic-shadow owner");
        Self { root }
    }

    pub(super) fn write(&self, path: &str, contents: &str) {
        fs::write(self.root.join(path), contents).expect("write generic-shadow fixture file");
    }

    pub(super) fn check(&self) -> Report {
        match build_lock(&self.root, "zrail.toml".as_ref()) {
            Ok(lock) => lock
                .write(&self.root.join("zrail.lock"))
                .expect("write generic-shadow lock"),
            Err(error) if error.to_string().contains("incomplete analysis") => {}
            Err(error) => panic!("build generic-shadow lock: {error}"),
        }
        check_repository(&self.root, "zrail.toml".as_ref(), "zrail.lock".as_ref())
            .expect("check generic-shadow fixture")
            .report
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("remove generic-shadow fixture");
        }
    }
}

pub(super) fn exact_owner_count(report: &Report, rule: &str, path: &str) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.id == "OWN-003"
                && finding.rule == rule
                && finding.path.as_deref() == Some(path)
                && finding.analysis == AnalysisQuality::Exact
        })
        .count()
}

pub(super) fn assert_no_owner(report: &Report, rule: &str, path: &str) {
    assert!(
        report.findings.iter().all(|finding| {
            !finding.id.starts_with("OWN-")
                || finding.rule != rule
                || finding.path.as_deref() != Some(path)
        }),
        "{}",
        report.human()
    );
}

pub(super) fn assert_complete(report: &Report) {
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.id != "RUST-INCLUDE-002"),
        "{}",
        report.human()
    );
}

pub(super) fn construction_owner(name: &str, selector: &str) -> String {
    format!(
        r#"[[owner]]
name = "{name}"
kind = "type-construction"
within = ["src/**"]
match = "{selector}"
allow = ["src/owner.rs"]
reason = "Construction stays behind the reviewed owner."
"#
    )
}

pub(super) fn call_owner(name: &str, selector: &str) -> String {
    format!(
        r#"[[owner]]
name = "{name}"
kind = "call"
within = ["src/**"]
match = "{selector}"
allow = ["src/owner.rs"]
reason = "Calls stay behind the reviewed owner."
"#
    )
}

const BASE_CONTRACT: &str = r#"schema = 1
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
"#;
