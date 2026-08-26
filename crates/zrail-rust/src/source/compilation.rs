//! Cargo compilation domains keep target namespaces distinct from policy reachability.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CompilationMode {
    Library,
    LibraryTest,
    Binary,
    BinaryTest,
    IntegrationTest,
    Benchmark,
    Example,
    ExampleTest,
    BuildScript,
}

impl CompilationMode {
    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::LibraryTest => "library-test",
            Self::Binary => "binary",
            Self::BinaryTest => "binary-test",
            Self::IntegrationTest => "integration-test",
            Self::Benchmark => "benchmark",
            Self::Example => "example",
            Self::ExampleTest => "example-test",
            Self::BuildScript => "build-script",
        }
    }

    pub(crate) const fn enables_cfg_test(self) -> bool {
        matches!(
            self,
            Self::LibraryTest
                | Self::BinaryTest
                | Self::IntegrationTest
                | Self::Benchmark
                | Self::ExampleTest
        )
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CompilationDomain {
    pub(crate) package: String,
    pub(crate) edition: String,
    pub(crate) target: String,
    pub(crate) mode: CompilationMode,
    pub(crate) feature_world: Option<String>,
    pub(crate) active_features: BTreeSet<String>,
}

impl CompilationDomain {
    pub(crate) fn canonical_identity(&self) -> String {
        format!(
            "package={};edition={};target={};mode={};feature-world={};features={}",
            self.package,
            self.edition,
            self.target,
            self.mode.canonical_name(),
            self.feature_world
                .as_deref()
                .unwrap_or("legacy-conditional"),
            canonical_features(&self.active_features),
        )
    }

    pub(crate) const fn has_exact_features(&self) -> bool {
        self.feature_world.is_some()
    }
}

fn canonical_features(features: &BTreeSet<String>) -> String {
    features
        .iter()
        .map(|feature| format!("{}:{feature}", feature.len()))
        .collect::<Vec<_>>()
        .join(",")
}
