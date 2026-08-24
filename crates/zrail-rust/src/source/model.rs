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
    ProductionOnly,
    Conditional,
    ConditionalTestOnly,
    ConditionalProductionOnly,
    Never,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GuardAvailability {
    Absent,
    Exact,
    Possible,
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
    pub(crate) namespace: FactNamespace,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FactNamespace {
    #[default]
    Unknown,
    Type,
    Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportBindingFact {
    pub(crate) name: Option<String>,
    pub(crate) target: String,
    pub(crate) kind: BindingKind,
    pub(crate) anchor: BindingAnchor,
    pub(crate) visibility: BindingVisibility,
    pub(crate) quality: AnalysisQuality,
    pub(crate) quality_without_macros: AnalysisQuality,
    pub(crate) replacement_macros: Vec<super::macro_binding_policy::MacroOccurrence>,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingKind {
    Import,
    Glob,
    TypeAlias,
    OpaqueAlias,
    Module(ModuleBinding),
    LocalType,
    LocalConstructor,
    LocalValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingAnchor {
    Lexical,
    UsePath,
    Absolute,
    ExternRoot,
    CrateRoot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModuleBinding {
    Inline(SourceSpan),
    External(SourceSpan),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BindingVisibility {
    Public,
    Private,
    Restricted(Vec<String>),
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
    pub(super) fn apply_guard(&mut self, guard: SyntaxGuard) {
        self.guard = self.guard.combine(guard);
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
        reachability.is_production() && self.guard.is_production_applicable()
    }

    pub(super) fn apply_guard(&mut self, guard: SyntaxGuard) {
        self.guard = self.guard.combine(guard);
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
    pub(crate) guard: SyntaxGuard,
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
    pub(crate) guard: SyntaxGuard,
    pub(crate) context: IncludeContext,
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) generic_types: Vec<String>,
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
    pub(crate) inline_module_scopes: Vec<SourceSpan>,
    pub(crate) compile_effects: Vec<CompileEffectFact>,
    pub(crate) lint_suppressions: Vec<ObservedFact>,
    pub(crate) unsafe_constructs: Vec<ObservedFact>,
    pub(crate) tests: Vec<ObservedFact>,
    pub(crate) modules: Vec<ModuleDeclaration>,
    pub(crate) includes: Vec<IncludeBoundary>,
    pub(crate) item_macros: Vec<ObservedFact>,
    pub(crate) opaque_binding_macros: Vec<ObservedFact>,
    pub(crate) facade_implementation: Vec<ObservedFact>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceIndex {
    pub(crate) files: Vec<RustFileFacts>,
    pub(crate) findings: Vec<Finding>,
}
