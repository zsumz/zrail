//! Crate-visible source facts form one declarative adapter boundary.

pub(crate) use super::{
    active_facts::retain_active_facts,
    associated_items::AssociatedItemFact,
    calls::resolution_finding as call_resolution_finding,
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
        AssociatedCandidateKind, AssociatedOccurrenceKind, AssociatedSegment, AsyncSyntaxFact,
        BindingAnchor, BindingKind, BindingVisibility, BoundSubject, CallResolutionFact,
        CallResolutionKind, ConstructorForm, FactNamespace, GenericArgumentsIdentity,
        GenericAssociatedCandidate, GenericPathIdentity, GenericRootIdentity, GenericRootShadow,
        GuardAvailability, ImplicitPreludeEligibility, ImportBindingFact, IncludeBoundary,
        IncludeContext, LexicalSelfIdentity, MacroDefinitionExport, MacroImportFact, ModuleBinding,
        ModuleDeclaration, ObservedFact, ProjectionIdentity, ProviderAuthority,
        RootLookupNamespace, RustFileFacts, SourceIndex, SourceSyntax, SyntaxGuard, TraitBoundFact,
        generic_root_identity, generic_root_shadow, identity_for_generic_root,
    },
    modules::{CompilationModuleEdge, ResolvedModuleEdge},
    operation_model::{SourceOperationFact, SourceOperationKind},
    parse::{fact_count, index_rust_source_with_hints},
    paths::{ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent},
    reachability::{Reachability, ReachabilityKind},
    source_instance::{
        CompilationRoot, SourceEntry, SourceInstanceId, SourceInstanceIssue, SourceInstances,
    },
    type_policy_index::inherit_replacing_mounts,
    type_policy_model::{
        DuplicationSyntaxKind, TraitImplFact, TraitImplPolarity, TypeDeclarationFact,
        TypeDeclarationKind,
    },
    type_shape::{ConstShapeFact, TypeArgumentFact, TypeShapeFact, type_shape},
};

#[cfg(test)]
pub(crate) use super::parse::index_rust_source;

#[cfg(test)]
pub(crate) use super::model::SourceAnalysisMetrics;

#[cfg(test)]
pub(crate) use super::operation_model::OperationSubjectOrigin;

#[cfg(test)]
pub(crate) use super::type_policy_model::TypePolicyFacts;
