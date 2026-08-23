//! One-pass Rust syntax facts shared by source and capability rails.

mod attributes;
mod calls;
mod canonical;
mod canonical_observed;
mod compilation;
mod compile_effect_model;
mod compile_effects;
mod depth;
mod fact;
mod import_aliases;
mod import_candidates;
mod import_helpers;
mod import_projection;
mod imports;
mod imports_collect;
mod include_alias_exports;
mod include_binding_catalog;
mod include_binding_expansion;
mod include_binding_helpers;
mod include_binding_lookup;
mod include_binding_missing;
mod include_binding_projection;
mod include_binding_resolution;
mod include_bindings;
mod include_edge;
mod include_glob_resolution;
mod include_module_identity;
mod include_namespace_completeness;
mod include_projection_apply;
mod include_projection_budget;
mod include_qualifiers;
mod include_resolution_state;
mod includes;
mod macro_builtin;
mod macro_definition_candidate;
mod macro_definition_resolution;
mod macro_definitions;
mod macro_expansion;
mod macro_inputs;
mod macro_model;
mod macro_model_access;
mod macro_origins;
mod macro_visibility;
mod macro_visibility_collect;
mod macro_visibility_graph;
mod macro_visibility_reachability;
mod model;
mod module_edge;
mod modules;
mod ordinary_binding_facts;
mod ordinary_bindings;
mod parse;
mod parse_facade;
mod paths;
mod reachability;
mod scoped_globs;
mod scoped_imports;
mod source_instance;
mod visitor;
mod visitor_attributes;
mod visitor_boundaries;
mod visitor_context;
mod visitor_imports;
mod visitor_init;
mod visitor_model;
mod visitor_paths;

pub(crate) use canonical::canonicalize as canonicalize_dependency_roots;
pub(crate) use compilation::{CompilationDomain, CompilationMode};
pub(crate) use compile_effect_model::CompileEffectFact;
pub(crate) use include_edge::{CompilationIncludeEdge, IncludeOccurrenceId};
pub(crate) use macro_model::{MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin};
pub(crate) use model::{
    BindingAnchor, BindingKind, BindingVisibility, FactNamespace, ImportBindingFact,
    IncludeBoundary, IncludeContext, MacroImportFact, ModuleBinding, ModuleDeclaration,
    ObservedFact, RustFileFacts, SourceIndex, SourceSyntax, SyntaxGuard,
};
pub(crate) use module_edge::{CompilationModuleEdge, ResolvedModuleEdge};
pub(crate) use parse::index_rust_source;
pub(crate) use paths::{
    ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent,
};
pub(crate) use reachability::{Reachability, ReachabilityKind};
pub(crate) use source_instance::{CompilationRoot, SourceEntry, SourceInstanceId, SourceInstances};
