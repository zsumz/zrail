//! Cargo compilation domains keep target namespaces distinct from policy reachability.

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
}

impl CompilationDomain {
    pub(crate) fn canonical_identity(&self) -> String {
        format!(
            "package={};edition={};target={};mode={}",
            self.package,
            self.edition,
            self.target,
            self.mode.canonical_name()
        )
    }
}
