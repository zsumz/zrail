//! Individual command implementations.

mod baseline;
mod baseline_edit;
mod baseline_output;
mod baseline_plan;
mod baseline_write;
mod check;
mod config_edit;
mod coverage;
mod diff;
mod doctor;
mod explain;
mod fmt;
mod git_base;
mod git_materialize;
mod git_process;
mod init;
mod init_preset;
mod init_template;
mod migrate_config;
mod migrate_lock;
mod migrate_lock_artifact;
mod mirrors;
mod mutation_paths;
mod result;
mod review;
mod update;
mod update_authority;

pub(crate) use baseline::baseline;
pub(crate) use check::check;
pub(crate) use coverage::coverage;
pub(crate) use diff::diff;
pub(crate) use doctor::doctor;
pub(crate) use explain::explain;
pub(crate) use fmt::format_config;
pub(crate) use init::init;
pub(crate) use migrate_config::migrate_config;
pub(crate) use migrate_lock::migrate_lock;
pub(crate) use mirrors::mirrors;
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
#[path = "update_contract_test.rs"]
mod update_contract_test;

#[cfg(test)]
#[path = "update_fixture_test.rs"]
mod update_fixture_test;
