//! Crate-visible source facts form one declarative adapter boundary.

pub(crate) use super::{
    active_facts::retain_active_facts,
    associated_items::AssociatedItemFact,
    canonical::canonicalize as canonicalize_dependency_roots,
    canonical_observed::CanonicalizationContext,
    cfg::{CfgContext, CfgPredicate, CfgTruth},
    compilation::{CompilationDomain, CompilationMode},
    compile_effects::CompileEffectFact,
    glob_imports::GlobImportFact,
    includes::{CompilationIncludeEdge, IncludeOccurrenceId},
    macro_binding_policy::BindingMacroPolicy,
    macro_model::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin},
    model::{
        AsyncSyntaxFact, BindingAnchor, BindingKind, BindingVisibility, CallResolutionFact,
        CallResolutionKind, ConstructorForm, FactNamespace, GuardAvailability,
        ImplicitPreludeEligibility, ImportBindingFact, IncludeBoundary, IncludeContext,
        MacroImportFact, ModuleBinding, ModuleDeclaration, ObservedFact, RustFileFacts,
        SourceIndex, SourceSyntax, SyntaxGuard,
    },
    modules::{CompilationModuleEdge, ResolvedModuleEdge},
    operation_model::{SourceOperationFact, SourceOperationKind},
    parse::{fact_count, index_rust_source},
    paths::{ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent},
    reachability::{Reachability, ReachabilityKind},
    source_instance::{
        CompilationRoot, SourceEntry, SourceInstanceId, SourceInstanceIssue, SourceInstances,
    },
    type_policy_model::{
        DuplicationSyntaxKind, TraitImplFact, TypeDeclarationFact, TypeDeclarationKind,
    },
    type_shape::{ConstShapeFact, TypeArgumentFact, TypeShapeFact, type_shape},
};

#[cfg(test)]
pub(crate) use super::model::SourceAnalysisMetrics;

#[cfg(test)]
pub(crate) use super::operation_model::OperationSubjectOrigin;

#[cfg(test)]
pub(crate) use super::type_policy_model::TypePolicyFacts;
