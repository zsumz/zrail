//! CLI parsing, command dispatch, and deterministic output.

mod args;
mod commands;
mod error;
mod output;
mod run;

pub(crate) use run::run;
