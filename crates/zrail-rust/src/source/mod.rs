//! One-pass Rust syntax facts shared by source and capability rails.

mod attributes;
mod calls;
mod canonical;
mod compile_effects;
mod depth;
mod fact;
mod import_aliases;
mod import_candidates;
mod imports;
mod includes;
mod macro_definitions;
mod macro_expansion;
mod macro_inputs;
mod model;
mod modules;
mod parse;
mod paths;
mod scoped_imports;
mod visitor;
mod visitor_attributes;
mod visitor_boundaries;
mod visitor_context;
mod visitor_imports;
mod visitor_init;
mod visitor_model;

pub(crate) use canonical::canonicalize as canonicalize_dependency_roots;
pub(crate) use model::{
    CompileEffectFact, IncludeBoundary, IncludeContext, ModuleDeclaration, ObservedFact,
    Reachability, RustFileFacts, SourceIndex, SourceSyntax,
};
pub(crate) use parse::index_rust_source;
pub(crate) use paths::{ModuleTarget, ResolutionError, join_relative, module_target, parent};
