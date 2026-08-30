//! Exact field representations render recursively from canonical path facts.

mod check;
mod compare;
mod render;
mod resolve;

pub(super) use check::check;
pub(crate) use compare::problems;
pub(crate) use resolve::resolve;
