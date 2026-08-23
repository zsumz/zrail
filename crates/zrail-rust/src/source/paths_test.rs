//! Rust module path tests cover defaults, attributes, inline modules, and escapes.

use zrail_core::SourceSpan;

use super::{ModuleTarget, SubmoduleBase, join_relative, module_target};
use crate::source::model::{InlineModulePath, ModuleDeclaration};

#[test]
fn module_paths_follow_rust_file_layout_rules() {
    let direct = declaration("worker");
    assert_eq!(
        module_target(
            "crates/demo/src/lib.rs",
            SubmoduleBase::SourceParent,
            &direct,
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/worker.rs".into(),
            nested: "crates/demo/src/worker/mod.rs".into(),
        })
    );
    assert_eq!(
        module_target(
            "crates/demo/src/worker.rs",
            SubmoduleBase::FileStemDirectory,
            &declaration("nested"),
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/worker/nested.rs".into(),
            nested: "crates/demo/src/worker/nested/mod.rs".into(),
        })
    );
}

#[test]
fn path_attributes_resolve_at_their_rust_defined_bases() {
    let mut renamed = declaration("worker");
    renamed.path = Some("alternate.rs".into());
    assert_eq!(
        module_target(
            "crates/demo/src/lib.rs",
            SubmoduleBase::SourceParent,
            &renamed,
        ),
        Ok(ModuleTarget::Exact("crates/demo/src/alternate.rs".into()))
    );
    renamed.inline_ancestors.push(InlineModulePath {
        name: "platform".into(),
        path: None,
        unresolved_path: false,
    });
    assert_eq!(
        module_target(
            "crates/demo/src/host.rs",
            SubmoduleBase::FileStemDirectory,
            &renamed,
        ),
        Ok(ModuleTarget::Exact(
            "crates/demo/src/host/platform/alternate.rs".into()
        ))
    );
}

#[test]
fn inline_path_attributes_replace_the_first_logical_component() {
    let mut declaration = declaration("tls");
    declaration.path = Some("local.rs".into());
    declaration.inline_ancestors.push(InlineModulePath {
        name: "thread".into(),
        path: Some("thread_files".into()),
        unresolved_path: false,
    });
    assert_eq!(
        module_target(
            "crates/demo/src/host.rs",
            SubmoduleBase::FileStemDirectory,
            &declaration,
        ),
        Ok(ModuleTarget::Exact(
            "crates/demo/src/thread_files/local.rs".into()
        ))
    );
}

#[test]
fn cargo_roots_are_mod_rs_regardless_of_their_file_name() {
    assert_eq!(
        module_target(
            "crates/demo/src/custom.rs",
            SubmoduleBase::SourceParent,
            &declaration("nested"),
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/nested.rs".into(),
            nested: "crates/demo/src/nested/mod.rs".into(),
        })
    );
    assert_eq!(
        module_target(
            "crates/demo/src/lib.rs",
            SubmoduleBase::FileStemDirectory,
            &declaration("nested"),
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/lib/nested.rs".into(),
            nested: "crates/demo/src/lib/nested/mod.rs".into(),
        })
    );
}

#[test]
fn exact_path_modules_give_children_the_loaded_sources_parent() {
    assert_eq!(
        module_target(
            "crates/demo/src/proxy.rs",
            SubmoduleBase::SourceParent,
            &declaration("child"),
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/child.rs".into(),
            nested: "crates/demo/src/child/mod.rs".into(),
        })
    );
    assert_eq!(
        module_target(
            "crates/demo/src/alt/proxy.rs",
            SubmoduleBase::SourceParent,
            &declaration("child"),
        ),
        Ok(ModuleTarget::Search {
            direct: "crates/demo/src/alt/child.rs".into(),
            nested: "crates/demo/src/alt/child/mod.rs".into(),
        })
    );
}

#[test]
fn lexical_escapes_and_platform_separators_are_rejected() {
    assert!(join_relative("crates/demo", "../../../outside.rs").is_err());
    assert!(join_relative("crates/demo", "src\\outside.rs").is_err());
}

fn declaration(name: &str) -> ModuleDeclaration {
    ModuleDeclaration {
        name: name.into(),
        path: None,
        cfg_test: false,
        unresolved_path: false,
        inline_ancestors: Vec::new(),
        lexical_scope: Vec::new(),
        span: Some(SourceSpan {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 1,
        }),
    }
}
