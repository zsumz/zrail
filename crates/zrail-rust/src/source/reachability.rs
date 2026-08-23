//! Execution-mode reachability shared by source-graph and policy projection.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ReachabilityKind {
    Production,
    Test,
    Benchmark,
    Example,
    Build,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Reachability(u8);

impl Reachability {
    const PRODUCTION_BIT: u8 = 1;
    const TEST_BIT: u8 = 1 << 1;
    const BENCHMARK_BIT: u8 = 1 << 2;
    const EXAMPLE_BIT: u8 = 1 << 3;
    const BUILD_BIT: u8 = 1 << 4;

    pub(crate) const UNREACHABLE: Self = Self(0);

    pub(crate) const fn from_kind(kind: ReachabilityKind) -> Self {
        Self(match kind {
            ReachabilityKind::Production => Self::PRODUCTION_BIT,
            ReachabilityKind::Test => Self::TEST_BIT,
            ReachabilityKind::Benchmark => Self::BENCHMARK_BIT,
            ReachabilityKind::Example => Self::EXAMPLE_BIT,
            ReachabilityKind::Build => Self::BUILD_BIT,
        })
    }

    pub(crate) const fn test() -> Self {
        Self::from_kind(ReachabilityKind::Test)
    }

    pub(crate) const fn is_unreachable(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub(crate) const fn covers(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) const fn is_test_only(self) -> bool {
        self.0 == Self::TEST_BIT
    }

    pub(crate) const fn contains(self, kind: ReachabilityKind) -> bool {
        self.0 & Self::from_kind(kind).0 != 0
    }

    pub(crate) const fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub(crate) const fn is_production(self) -> bool {
        self.contains(ReachabilityKind::Production)
    }

    pub(crate) const fn is_non_test_target(self) -> bool {
        self.contains(ReachabilityKind::Production)
            || self.contains(ReachabilityKind::Benchmark)
            || self.contains(ReachabilityKind::Example)
            || self.contains(ReachabilityKind::Build)
    }

    pub(crate) fn name(self) -> String {
        match self.0 {
            0 => "unreachable".into(),
            Self::TEST_BIT => "test-only".into(),
            Self::PRODUCTION_BIT => "production".into(),
            value if value == Self::PRODUCTION_BIT | Self::TEST_BIT => "both".into(),
            _ => [
                (ReachabilityKind::Production, "production"),
                (ReachabilityKind::Test, "test"),
                (ReachabilityKind::Benchmark, "benchmark"),
                (ReachabilityKind::Example, "example"),
                (ReachabilityKind::Build, "build"),
            ]
            .into_iter()
            .filter_map(|(kind, name)| self.contains(kind).then_some(name))
            .collect::<Vec<_>>()
            .join(","),
        }
    }
}
