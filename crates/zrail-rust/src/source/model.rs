//! Normalized facts extracted from one Rust source file.

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::inventory::FileClass;

use super::macro_model::{CompileEffectFact, MacroExpansionFact};

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

    pub(crate) const fn is_test_only(self) -> bool {
        self.0 == Self::TEST_BIT
    }

    pub(crate) const fn contains(self, kind: ReachabilityKind) -> bool {
        self.0 & Self::from_kind(kind).0 != 0
    }
}

impl Reachability {
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SyntaxGuard {
    #[default]
    Ordinary,
    TestOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedFact {
    pub(crate) name: String,
    pub(crate) canonical: Vec<String>,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MacroImportFact {
    pub(crate) name: String,
    pub(crate) target: String,
    pub(crate) quality: AnalysisQuality,
    pub(crate) re_export: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroDefinitionFact {
    pub(crate) name: String,
    pub(crate) sha256: String,
    pub(crate) span: Option<SourceSpan>,
}

impl ObservedFact {
    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.canonical
            .iter()
            .map(String::as_str)
            .chain(self.canonical.is_empty().then_some(self.name.as_str()))
    }

    pub(crate) const fn is_production_applicable(&self, reachability: Reachability) -> bool {
        reachability.is_production() && matches!(self.guard, SyntaxGuard::Ordinary)
    }

    pub(super) fn mark_test_only(&mut self) {
        self.guard = SyntaxGuard::TestOnly;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InlineModulePath {
    pub(crate) name: String,
    pub(crate) path: Option<String>,
    pub(crate) unresolved_path: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModuleDeclaration {
    pub(crate) name: String,
    pub(crate) path: Option<String>,
    pub(crate) cfg_test: bool,
    pub(crate) unresolved_path: bool,
    pub(crate) inline_ancestors: Vec<InlineModulePath>,
    pub(crate) span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceSyntax {
    Items,
    Expression,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncludeContext {
    Items,
    Expression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IncludeBoundary {
    pub(crate) path: Option<String>,
    pub(crate) out_dir: Option<String>,
    pub(crate) expression: String,
    pub(crate) generated: bool,
    pub(crate) cfg_test: bool,
    pub(crate) context: IncludeContext,
    pub(crate) span: Option<SourceSpan>,
}

#[derive(Clone, Debug)]
pub(crate) struct RustFileFacts {
    pub(crate) relative: String,
    pub(crate) packages: Vec<String>,
    pub(crate) class: FileClass,
    pub(crate) reachability: Reachability,
    pub(crate) syntax: SourceSyntax,
    pub(crate) lines: usize,
    pub(crate) module_docs: bool,
    pub(crate) paths: Vec<ObservedFact>,
    pub(crate) calls: Vec<ObservedFact>,
    pub(crate) methods: Vec<ObservedFact>,
    pub(crate) macros: Vec<ObservedFact>,
    pub(crate) macro_imports: Vec<MacroImportFact>,
    pub(crate) macro_expansions: Vec<MacroExpansionFact>,
    pub(crate) opaque_macro_inputs: Vec<MacroExpansionFact>,
    pub(crate) macro_definitions: Vec<MacroDefinitionFact>,
    pub(crate) compile_effects: Vec<CompileEffectFact>,
    pub(crate) lint_suppressions: Vec<ObservedFact>,
    pub(crate) unsafe_constructs: Vec<ObservedFact>,
    pub(crate) tests: Vec<ObservedFact>,
    pub(crate) modules: Vec<ModuleDeclaration>,
    pub(crate) includes: Vec<IncludeBoundary>,
    pub(crate) item_macros: Vec<ObservedFact>,
    pub(crate) facade_implementation: Vec<ObservedFact>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceIndex {
    pub(crate) files: Vec<RustFileFacts>,
    pub(crate) findings: Vec<Finding>,
}
