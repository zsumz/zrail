//! Logical macro namespaces resolve exports before legacy visibility fallback.

#[path = "macro_exports.rs"]
mod exports;
#[path = "logical_modules.rs"]
mod logical_modules;
#[path = "macro_visibility.rs"]
mod visibility;

#[cfg(test)]
use super::ReachabilityKind;
use super::{
    BindingKind, BindingVisibility, CompilationDomain, CompilationMode, GuardAvailability,
    IncludeContext, MacroCandidate, MacroDefinitionExport, MacroDerivation, MacroExpansionFact,
    MacroImportFact, MacroOrigin, ModuleBinding, ObservedFact, Reachability, RustFileFacts,
    SourceEntry, SourceIndex, SourceInstanceId, SourceInstances, SourceSyntax, SyntaxGuard,
    macro_visibility_graph, source_instance,
};

pub(super) use exports::MacroExports;
pub(super) use visibility::{MacroVisibility, repository_path};
