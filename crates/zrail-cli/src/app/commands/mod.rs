//! Individual command implementations.

mod check;
mod diff;
mod doctor;
mod explain;
mod git_base;
mod git_materialize;
mod git_process;
mod init;
mod init_template;
mod result;
mod update;

pub(crate) use check::check;
pub(crate) use diff::diff;
pub(crate) use doctor::doctor;
pub(crate) use explain::explain;
pub(crate) use init::init;
pub(crate) use result::CommandResult;
pub(crate) use update::update;

#[cfg(test)]
#[path = "git_base_migration_test.rs"]
mod git_base_migration_test;

#[cfg(test)]
#[path = "update_contract_test.rs"]
mod update_contract_test;
