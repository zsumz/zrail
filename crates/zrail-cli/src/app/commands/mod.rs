//! Individual command implementations.

mod baseline;
mod baseline_edit;
mod baseline_output;
mod baseline_plan;
mod baseline_write;
mod check;
mod diff;
mod doctor;
mod explain;
mod git_base;
mod git_materialize;
mod git_process;
mod init;
mod init_preset;
mod init_template;
mod mutation_paths;
mod result;
mod review;
mod update;
mod update_authority;

pub(crate) use baseline::baseline;
pub(crate) use check::check;
pub(crate) use diff::diff;
pub(crate) use doctor::doctor;
pub(crate) use explain::explain;
pub(crate) use init::init;
pub(crate) use result::CommandResult;
pub(crate) use review::review;
pub(crate) use update::update;

#[cfg(test)]
#[path = "git_base_migration_test.rs"]
mod git_base_migration_test;

#[cfg(test)]
#[path = "review_dependency_test.rs"]
mod review_dependency_test;

#[cfg(test)]
mod review_fixture;

#[cfg(test)]
#[path = "review_lock_test.rs"]
mod review_lock_test;

#[cfg(test)]
mod mutation_paths_test;

#[cfg(test)]
#[path = "update_contract_test.rs"]
mod update_contract_test;

#[cfg(test)]
#[path = "update_fixture_test.rs"]
mod update_fixture_test;
