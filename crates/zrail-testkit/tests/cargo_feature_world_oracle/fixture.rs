//! Temporary resolver fixtures keep Cargo execution outside the runtime analyzer.

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "fixture/manifests.rs"]
mod manifests;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

pub(super) struct Fixture {
    pub(super) root: PathBuf,
    scenario: Scenario,
}

impl Fixture {
    pub(super) fn new(resolver: &str, scenario: Scenario) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "zrail-feature-oracle-{}-{resolver}-{scenario:?}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        for member in scenario.members() {
            fs::create_dir_all(root.join(member).join("src")).expect("create member source");
            fs::write(root.join(member).join("src/lib.rs"), "//! Test library.\n")
                .expect("write member library");
        }
        fs::write(
            root.join("Cargo.toml"),
            manifests::workspace(resolver, scenario),
        )
        .expect("write workspace manifest");
        fs::write(root.join("app/Cargo.toml"), manifests::app(scenario))
            .expect("write app manifest");
        fs::write(root.join("shared/Cargo.toml"), manifests::shared(scenario))
            .expect("write shared manifest");
        if scenario.has_helper() {
            fs::write(root.join("helper/Cargo.toml"), manifests::helper(scenario))
                .expect("write helper manifest");
        }
        if scenario.has_proc_macro() {
            fs::write(root.join("macros/Cargo.toml"), manifests::PROC_MACRO)
                .expect("write proc-macro manifest");
        }
        fs::write(
            root.join("app/build.rs"),
            "//! Build script.\nfn main() {}\n",
        )
        .expect("write build script");
        fs::write(root.join("zrail.toml"), manifests::contract(scenario))
            .expect("write zrail contract");
        Self { root, scenario }
    }

    pub(super) fn cargo_feature_sets(&self) -> BTreeSet<Vec<String>> {
        let mut command = Command::new("cargo");
        command
            .arg("tree")
            .arg("--manifest-path")
            .arg(self.root.join("Cargo.toml"))
            .args([
                "-p",
                "oracle-app",
                "--edges",
                "normal,build",
                "--format",
                "{p}|{f}",
                "--no-dedupe",
                "--offline",
                "--color",
                "never",
            ]);
        if let Some(feature) = self.scenario.cargo_authored_feature() {
            command.args(["--features", feature]);
        }
        let output = command.output().expect("run trusted Cargo feature oracle");
        assert!(
            output.status.success(),
            "cargo tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("Cargo output is UTF-8")
            .lines()
            .filter(|line| line.contains("oracle-shared v0.0.0"))
            .filter_map(|line| line.rsplit_once('|').map(|(_, features)| features))
            .map(|features| {
                features
                    .split(',')
                    .filter(|feature| !feature.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Scenario {
    EmptyDirect,
    HostOnlyDirect,
    TargetOnlyDirect,
    MaskedDirect,
    SelectedDirect,
    DefaultDirect,
    EmptyTransitive,
    HostOnlyTransitive,
    MaskedTransitive,
    EmptyProcMacro,
    HostOnlyProcMacro,
    MaskedProcMacro,
}

impl Scenario {
    fn members(self) -> &'static [&'static str] {
        match self.family() {
            Family::Direct => &["app", "shared"],
            Family::Transitive => &["app", "shared", "helper"],
            Family::ProcMacro => &["app", "shared", "helper", "macros"],
        }
    }

    const fn family(self) -> Family {
        match self {
            Self::EmptyDirect
            | Self::HostOnlyDirect
            | Self::TargetOnlyDirect
            | Self::MaskedDirect
            | Self::SelectedDirect
            | Self::DefaultDirect => Family::Direct,
            Self::EmptyTransitive | Self::HostOnlyTransitive | Self::MaskedTransitive => {
                Family::Transitive
            }
            Self::EmptyProcMacro | Self::HostOnlyProcMacro | Self::MaskedProcMacro => {
                Family::ProcMacro
            }
        }
    }

    const fn target_feature(self) -> bool {
        matches!(
            self,
            Self::TargetOnlyDirect
                | Self::MaskedDirect
                | Self::MaskedTransitive
                | Self::MaskedProcMacro
        )
    }

    const fn host_feature(self) -> bool {
        matches!(
            self,
            Self::HostOnlyDirect
                | Self::MaskedDirect
                | Self::HostOnlyTransitive
                | Self::MaskedTransitive
                | Self::HostOnlyProcMacro
                | Self::MaskedProcMacro
        )
    }

    const fn has_helper(self) -> bool {
        !matches!(self.family(), Family::Direct)
    }

    const fn has_proc_macro(self) -> bool {
        matches!(self.family(), Family::ProcMacro)
    }

    const fn cargo_authored_feature(self) -> Option<&'static str> {
        match self {
            Self::SelectedDirect => Some("oracle-shared/context"),
            Self::DefaultDirect => Some("oracle-shared/default"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum Family {
    Direct,
    Transitive,
    ProcMacro,
}
