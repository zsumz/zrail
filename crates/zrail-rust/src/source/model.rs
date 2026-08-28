//! Normalized facts extracted from one Rust source file.

#[path = "model_bindings.rs"]
mod bindings;
#[path = "model_generic_roots.rs"]
mod generic_roots;
#[path = "source_metrics.rs"]
mod source_metrics;

use zrail_core::{AnalysisQuality, Finding, SourceSpan};

use crate::inventory::FileClass;

use super::Reachability;
use super::{CompileEffectFact, IncludeOccurrenceId, macro_model::MacroExpansionFact};

pub(crate) use super::include_bindings::implicit_prelude::PreludeDirective;
pub(crate) use bindings::{
    BindingAnchor, BindingKind, BindingVisibility, ConstructorForm, ImportBindingFact,
    MacroImportFact, ModuleBinding,
};
pub(crate) use generic_roots::{
    GenericRootIdentity, GenericRootShadow, RootLookupNamespace, generic_root_identity,
    generic_root_shadow, identity_for_generic_root,
};

pub(crate) use source_metrics::SourceAnalysisMetrics;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AsyncSyntaxFact {
    pub(crate) kind: zrail_core::AsyncSyntax,
    pub(crate) observation: ObservedFact,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SyntaxGuard {
    #[default]
    Ordinary,
    TestOnly,
    ProductionOnly,
    Never,
    Predicate(super::CfgPredicate),
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
    pub(crate) implicit_prelude: ImplicitPreludeEligibility,
    pub(crate) canonical: Vec<String>,
    pub(crate) span: Option<SourceSpan>,
    pub(crate) quality: AnalysisQuality,
    pub(crate) guard: SyntaxGuard,
    pub(crate) lexical_scope: Vec<SourceSpan>,
    pub(crate) namespace: FactNamespace,
    pub(crate) generic_shadow: Option<GenericRootShadow>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ImplicitPreludeEligibility {
    Eligible,
    Disabled,
    LocalShadow,
    GenericShadow,
    PossibleShadow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallResolutionFact {
    pub(crate) written: String,
    pub(crate) span: SourceSpan,
    pub(crate) guard: SyntaxGuard,
    pub(crate) kind: CallResolutionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallResolutionKind {
    AssociatedTypeProjection,
    ExplicitTrait,
    GenericAssociatedItem,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum FactNamespace {
    #[default]
    Unknown,
    Type,
    Value,
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
    pub(super) fn apply_guard(&mut self, guard: &SyntaxGuard) {
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

    pub(crate) fn is_production_applicable(&self, reachability: Reachability) -> bool {
        reachability.is_production() && self.guard.is_production_applicable()
    }

    pub(super) fn apply_guard(&mut self, guard: &SyntaxGuard) {
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
    pub(crate) generic_values: Vec<String>,
    pub(crate) value_shadows: Vec<(String, SyntaxGuard)>,
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
    pub(crate) call_resolutions: Vec<CallResolutionFact>,
    pub(crate) methods: Vec<ObservedFact>,
    pub(crate) operations: Vec<super::operation_model::SourceOperationFact>,
    pub(crate) macros: Vec<ObservedFact>,
    pub(crate) macro_imports: Vec<MacroImportFact>,
    pub(crate) macro_expansions: Vec<MacroExpansionFact>,
    pub(crate) opaque_macro_inputs: Vec<MacroExpansionFact>,
    pub(crate) macro_definitions: Vec<MacroDefinitionFact>,
    pub(crate) import_bindings: Vec<ImportBindingFact>,
    pub(crate) associated_items: Vec<super::associated_items::AssociatedItemFact>,
    pub(crate) glob_imports: Vec<super::glob_imports::GlobImportFact>,
    pub(crate) inline_module_scopes: Vec<SourceSpan>,
    pub(crate) prelude_directives: Vec<PreludeDirective>,
    pub(crate) compile_effects: Vec<CompileEffectFact>,
    pub(crate) lint_suppressions: Vec<ObservedFact>,
    pub(crate) unsafe_constructs: Vec<ObservedFact>,
    pub(crate) async_syntax: Vec<AsyncSyntaxFact>,
    pub(crate) type_policy: super::type_policy_model::TypePolicyFacts,
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
    pub(crate) analysis_metrics: SourceAnalysisMetrics,
}
