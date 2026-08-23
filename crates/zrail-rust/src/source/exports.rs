//! Crate-visible source facts form one declarative adapter boundary.

pub(crate) use super::{
    canonical::canonicalize as canonicalize_dependency_roots,
    canonical_context::CanonicalizationContext,
    compilation::{CompilationDomain, CompilationMode},
    compile_effect_model::CompileEffectFact,
    include_edge::{CompilationIncludeEdge, IncludeOccurrenceId},
    macro_binding_policy::BindingMacroPolicy,
    macro_model::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin},
    model::{
        BindingAnchor, BindingKind, BindingVisibility, FactNamespace, ImportBindingFact,
        IncludeBoundary, IncludeContext, MacroImportFact, ModuleBinding, ModuleDeclaration,
        ObservedFact, RustFileFacts, SourceIndex, SourceSyntax, SyntaxGuard,
    },
    module_edge::{CompilationModuleEdge, ResolvedModuleEdge},
    parse::index_rust_source,
    paths::{ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent},
    reachability::{Reachability, ReachabilityKind},
    source_instance::{CompilationRoot, SourceEntry, SourceInstanceId, SourceInstances},
};
