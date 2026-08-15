//! Normalized facts extracted from one Rust source file.

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::inventory::FileClass;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Reachability {
    #[default]
    Unreachable,
    TestOnly,
    Production,
    Both,
}

impl Reachability {
    pub(crate) const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unreachable, value) | (value, Self::Unreachable) => value,
            (Self::TestOnly, Self::TestOnly) => Self::TestOnly,
            (Self::Production, Self::Production) => Self::Production,
            _ => Self::Both,
        }
    }

    pub(crate) const fn is_production(self) -> bool {
        matches!(self, Self::Production | Self::Both)
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Unreachable => "unreachable",
            Self::TestOnly => "test-only",
            Self::Production => "production",
            Self::Both => "both",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedFact {
    pub(crate) name: String,
    pub(crate) canonical: Vec<String>,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) quality: AnalysisQuality,
}

impl ObservedFact {
    pub(crate) fn policy_names(&self) -> impl Iterator<Item = &str> {
        self.canonical
            .iter()
            .map(String::as_str)
            .chain(self.canonical.is_empty().then_some(self.name.as_str()))
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
