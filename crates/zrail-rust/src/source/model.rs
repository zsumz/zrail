//! Normalized facts extracted from one Rust source file.

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::inventory::FileClass;

use super::Reachability;
use super::{CompileEffectFact, IncludeOccurrenceId, macro_model::MacroExpansionFact};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SyntaxGuard {
    #[default]
    Ordinary,
    TestOnly,
}

impl SyntaxGuard {
    pub(crate) const fn for_test_only(test_only: bool) -> Self {
        if test_only {
            Self::TestOnly
        } else {
            Self::Ordinary
        }
    }

    pub(crate) const fn available_in(self, context: Self) -> bool {
        matches!(self, Self::Ordinary) || matches!(context, Self::TestOnly)
    }

    pub(crate) const fn combine(self, other: Self) -> Self {
        if matches!(self, Self::TestOnly) || matches!(other, Self::TestOnly) {
            Self::TestOnly
        } else {
            Self::Ordinary
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedFact {
    pub(crate) name: String,
    pub(crate) written: Option<String>,
    pub(crate) canonical: Vec<String>,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportBindingFact {
    pub(crate) name: Option<String>,
    pub(crate) target: String,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MacroImportFact {
    pub(crate) name: String,
    pub(crate) target: String,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) re_export: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MacroDefinitionFact {
    pub(crate) name: String,
    pub(crate) sha256: String,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

impl MacroDefinitionFact {
    pub(super) fn mark_test_only(&mut self) {
        self.guard = SyntaxGuard::TestOnly;
    }
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
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceSyntax {
    Items,
    Expression,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) occurrence: IncludeOccurrenceId,
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
    pub(crate) import_bindings: Vec<ImportBindingFact>,
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
