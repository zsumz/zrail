//! Exact field representations render recursively from canonical path facts.

mod check;
mod render;

pub(super) use check::check;
pub(crate) use render::{render_contract, render_source};
