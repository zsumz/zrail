//! Visitor support is grouped behind its traversal façade without widening it.

use super::{
    AsyncSyntaxFact, CompileEffectFact, FactNamespace, GlobImportFact, GuardAvailability,
    MacroCandidate, MacroDerivation, MacroExpansionFact, ObservedFact, SourceOperationFact,
    SourceOperationKind, SyntaxGuard, attributes, calls, fact, glob_imports, import_helpers,
    imports, includes, macro_expansion, macro_origins, model, operation_model, ordinary_bindings,
    place_expression, scoped_globs, scoped_imports,
};

#[path = "visitor_async.rs"]
pub(in crate::source) mod visitor_async;
#[path = "visitor_attributes.rs"]
pub(in crate::source) mod visitor_attributes;
#[path = "visitor_boundaries.rs"]
pub(in crate::source) mod visitor_boundaries;
#[path = "visitor_calls.rs"]
pub(in crate::source) mod visitor_calls;
#[path = "visitor_context.rs"]
pub(in crate::source) mod visitor_context;
#[path = "visitor_field_operations.rs"]
pub(in crate::source) mod visitor_field_operations;
#[path = "visitor_imports.rs"]
pub(in crate::source) mod visitor_imports;
#[path = "visitor_init.rs"]
pub(in crate::source) mod visitor_init;
#[path = "visitor_model.rs"]
pub(in crate::source) mod visitor_model;
#[path = "visitor_operations.rs"]
pub(in crate::source) mod visitor_operations;
#[path = "visitor_paths.rs"]
pub(in crate::source) mod visitor_paths;
#[path = "visitor_values.rs"]
pub(in crate::source) mod visitor_values;

pub(in crate::source) use visitor_model::FactVisitor;
