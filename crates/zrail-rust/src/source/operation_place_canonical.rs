//! Projected type declarations repair exact field places across source files.

mod catalog;
mod repair;
mod routes;

pub(super) use repair::apply;
