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
    pub(crate) target: String,
    pub(crate) mode: CompilationMode,
}
