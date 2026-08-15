//! Normalized facts extracted from one Rust source file.

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::inventory::FileClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedFact {
    pub(crate) name: String,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) quality: AnalysisQuality,
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
    pub(crate) context: IncludeContext,
    pub(crate) span: Option<SourceSpan>,
}

#[derive(Clone, Debug)]
pub(crate) struct RustFileFacts {
    pub(crate) relative: String,
    pub(crate) class: FileClass,
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
