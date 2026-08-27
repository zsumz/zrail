//! Import and ordinary-binding facts shared by source resolution passes.

use zrail_core::{AnalysisQuality, SourceSpan};

use super::super::macro_binding_policy::MacroOccurrence;
use super::SyntaxGuard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportBindingFact {
    pub(crate) name: Option<String>,
    pub(crate) target: String,
    pub(crate) kind: BindingKind,
    pub(crate) anchor: BindingAnchor,
    pub(crate) visibility: BindingVisibility,
    pub(crate) quality: AnalysisQuality,
    pub(crate) quality_without_macros: AnalysisQuality,
    pub(crate) replacement_macros: Vec<MacroOccurrence>,
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
    LocalConstructor(ConstructorForm),
    LocalValue,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ConstructorForm {
    Named,
    Tuple,
    Unit,
    Unknown,
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
