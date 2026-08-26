//! Cargo roots and Rust source edges must form one closed, analyzable graph.

mod api;
mod boundary;
mod compilation;
mod diagnostics;
mod external_module;
mod feature_worlds;
mod include;
mod item_macros;
mod model;
mod walker;

use walker::{TraversalContext, Walker};

pub(crate) use api::{
    analyze, item_macro_authorities, item_macro_is_authorized, item_macro_selector,
    review_item_macros,
};
pub(crate) use model::SourceGraphAnalysis;
