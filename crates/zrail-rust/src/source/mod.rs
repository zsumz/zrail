//! One-pass Rust syntax facts shared by source and capability rails.

mod attributes;
mod calls;
mod depth;
mod fact;
mod import_candidates;
mod imports;
mod includes;
mod model;
mod modules;
mod parse;
mod paths;
mod visitor;
mod visitor_boundaries;

pub(crate) use model::{
    IncludeBoundary, IncludeContext, ModuleDeclaration, ObservedFact, RustFileFacts, SourceIndex,
    SourceSyntax,
};
pub(crate) use parse::index_rust_source;
pub(crate) use paths::{ModuleTarget, ResolutionError, join_relative, module_target, parent};
