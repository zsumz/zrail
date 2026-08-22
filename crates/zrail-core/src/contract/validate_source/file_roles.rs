//! Exact source-role overrides are bounded, reasoned, and handwritten.

use std::collections::BTreeSet;

use super::super::{
    Contract, validate_limits::ValidationErrors, validate_paths::validate_repository_literal,
    validate_sets::require_reason,
};

pub(super) fn validate(contract: &Contract, errors: &mut ValidationErrors) {
    let mut paths = BTreeSet::new();
    for role in &contract.source.rust.file_roles {
        validate_repository_literal(&role.path, errors);
        require_reason("file-role override", &role.path, &role.reason, errors);
        if !std::path::Path::new(&role.path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
        {
            errors.push(format!(
                "file-role override must name .rs source: {:?}",
                role.path
            ));
        }
        if !paths.insert(role.path.as_str()) {
            errors.push(format!("duplicate file-role override for {:?}", role.path));
        }
        if contract
            .source
            .rust
            .generated
            .iter()
            .any(|generated| contains(&generated.root, &role.path))
        {
            errors.push(format!(
                "generated source may not have a file-role override: {:?}",
                role.path
            ));
        }
    }
}

fn contains(root: &str, path: &str) -> bool {
    root == "." || path == root || path.starts_with(&format!("{root}/"))
}

#[cfg(test)]
#[path = "file_roles_test.rs"]
mod file_roles_test;
