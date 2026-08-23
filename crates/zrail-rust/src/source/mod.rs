//! One-pass Rust syntax facts shared by source and capability rails.

mod attributes;
mod calls;
mod canonical;
mod canonical_observed;
mod compile_effects;
mod depth;
mod fact;
mod import_aliases;
mod import_candidates;
mod import_helpers;
mod import_projection;
mod imports;
mod imports_collect;
mod includes;
mod macro_builtin;
mod macro_definitions;
mod macro_expansion;
mod macro_inputs;
mod macro_model;
mod macro_origins;
mod macro_visibility;
mod macro_visibility_collect;
mod macro_visibility_graph;
mod macro_visibility_reachability;
mod model;
mod module_edge;
mod modules;
mod parse;
mod parse_facade;
mod paths;
mod reachability;
mod scoped_globs;
mod scoped_imports;
mod visitor;
mod visitor_attributes;
mod visitor_boundaries;
mod visitor_context;
mod visitor_imports;
mod visitor_init;
mod visitor_model;

pub(crate) use canonical::canonicalize as canonicalize_dependency_roots;
pub(crate) use macro_model::{
    CompileEffectFact, MacroCandidate, MacroDerivation, MacroExpansionFact, MacroOrigin,
};
pub(crate) use model::{
    IncludeBoundary, IncludeContext, MacroImportFact, ModuleDeclaration, ObservedFact,
    RustFileFacts, SourceIndex, SourceSyntax, SyntaxGuard,
};
pub(crate) use module_edge::ResolvedModuleEdge;
pub(crate) use parse::index_rust_source;
pub(crate) use paths::{
    ModuleTarget, ResolutionError, SubmoduleBase, join_relative, module_target, parent,
};
pub(crate) use reachability::{Reachability, ReachabilityKind};
