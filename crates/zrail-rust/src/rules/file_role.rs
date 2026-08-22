//! Exact source-role overrides remain live, nonredundant, and role-bounded.

use zrail_core::{FileRole, Finding, FindingSink};

use crate::inventory::FileClass;

use super::RuleContext;

pub(super) fn evaluate(context: &RuleContext<'_>, findings: &mut FindingSink) {
    for declared in &context.contract.source.rust.file_roles {
        let file = context
            .source
            .files
            .iter()
            .find(|file| file.relative == declared.path && !file.reachability.is_unreachable());
        let Some(file) = file else {
            findings.push(
                Finding::error(
                    "RUST-ROLE-001",
                    "rust.file-role",
                    "source-shape",
                    format!(
                        "file-role override names missing reachable Rust source {:?}",
                        declared.path
                    ),
                )
                .at(&declared.path, None)
                .because(&declared.reason),
            );
            continue;
        };
        if !matches!(file.class, FileClass::Facade | FileClass::Implementation) {
            findings.push(
                Finding::error(
                    "RUST-ROLE-002",
                    "rust.file-role",
                    "source-shape",
                    format!(
                        "inferred {} source may not be reclassified",
                        crate::source_policy::role_name(file.class)
                    ),
                )
                .at(&file.relative, None)
                .because(&declared.reason),
            );
            continue;
        }
        let declared_class = match declared.role {
            FileRole::Facade => FileClass::Facade,
            FileRole::Implementation => FileClass::Implementation,
        };
        if declared_class == file.class {
            findings.push(
                Finding::error(
                    "RUST-ROLE-003",
                    "rust.file-role",
                    "source-shape",
                    format!(
                        "file-role override redundantly declares inferred {} role",
                        crate::source_policy::role_name(file.class)
                    ),
                )
                .at(&file.relative, None)
                .because(&declared.reason)
                .with_help("remove the stale file-role override"),
            );
        }
    }
}
