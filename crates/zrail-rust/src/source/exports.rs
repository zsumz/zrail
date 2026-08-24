//! Crate-visible source facts form one declarative adapter boundary.

pub(crate) use super::{
    canonical::canonicalize as canonicalize_dependency_roots,
    canonical_observed::CanonicalizationContext,
    compilation::{CompilationDomain, CompilationMode},
    compile_effects::CompileEffectFact,
    includes::{CompilationIncludeEdge, IncludeOccurrenceId},
    macro_binding_policy::BindingMacroPolicy,
    macro_model::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin},
    model::{
        BindingAnchor, BindingKind, BindingVisibility, FactNamespace, GuardAvailability,
        ImportBindingFact, IncludeBoundary, IncludeContext, MacroImportFact, ModuleBinding,
        ModuleDeclaration, ObservedFact, RustFileFacts, SourceIndex, SourceSyntax, SyntaxGuard,
    },
    modules::{CompilationModuleEdge, ResolvedModuleEdge},
    operation_model::{SourceOperationFact, SourceOperationKind},
    parse::{fact_count, index_rust_source},
    paths::{ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent},
    reachability::{Reachability, ReachabilityKind},
    source_instance::{
        CompilationRoot, SourceEntry, SourceInstanceId, SourceInstanceIssue, SourceInstances,
    },
};

#[cfg(test)]
pub(crate) use super::model::SourceAnalysisMetrics;
