//! Qualified module paths retain include-spliced ordinary bindings.

use super::{
    PRODUCTION_OWNER, assert_no_owned_call, assert_owned_call, check, fixture, reset, write,
    write_executor, write_lock,
};

#[test]
fn included_alias_via_self_cannot_bypass_call_ownership() {
    let root = fixture("qualified-self", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\npub fn hidden() { let _ = self::Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn included_alias_via_crate_cannot_bypass_call_ownership() {
    let root = fixture("qualified-crate", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\npub fn hidden() { let _ = crate::Spawn::new(\"sh\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn parent_included_alias_via_super_cannot_bypass_call_ownership() {
    let root = fixture("qualified-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"imports.rs\");\nmod child;\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write(
        &root,
        "src/child.rs",
        "//! Child.\npub fn hidden() { let _ = super::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/child.rs");
    reset(&root);
}

#[test]
fn super_skips_the_include_edge_before_leaving_a_module() {
    let root = fixture("qualified-super-include", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod outer { use std::process::Command as Spawn; mod inner { include!(\"qualified_body.rs\"); } }\n",
    );
    write(
        &root,
        "src/qualified_body.rs",
        "pub fn hidden() { let _ = super::Spawn::new(\"sh\"); }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/qualified_body.rs");
    reset(&root);
}

#[test]
fn super_leaves_an_inline_module_declared_inside_an_include() {
    let root = fixture("qualified-included-inline", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\ninclude!(\"included_module.rs\");\n",
    );
    write(
        &root,
        "src/included_module.rs",
        "use std::process::Command as Spawn; mod inner { pub fn hidden() { let _ = super::Spawn::new(\"sh\"); } }\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/included_module.rs");
    reset(&root);
}

#[test]
fn nested_self_super_uses_the_effective_parent_module() {
    let root = fixture("qualified-self-super", "");
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\nmod outer { include!(\"imports.rs\"); mod inner { pub fn hidden() { let _ = self::super::Spawn::new(\"sh\"); } } }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    assert_owned_call(&check(&root), "process-spawn", "src/lib.rs");
    reset(&root);
}

#[test]
fn qualified_alias_in_the_allowed_owner_remains_exact() {
    let root = fixture("qualified-allowed", "");
    write(&root, "src/lib.rs", "//! Library.\nmod executor;\n");
    write(
        &root,
        "src/executor.rs",
        "//! Authorized process owner.\ninclude!(\"imports.rs\");\npub fn allowed() { let _ = self::Spawn::new(\"true\"); }\n",
    );
    write(
        &root,
        "src/imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_lock(&root);

    let report = check(&root);
    assert_no_owned_call(&report, "process-spawn", "src/executor.rs");
    reset(&root);
}

#[test]
fn test_only_qualified_alias_does_not_affect_production() {
    let root = fixture("qualified-test", PRODUCTION_OWNER);
    write(
        &root,
        "src/lib.rs",
        "//! Library.\nmod executor;\n#[cfg(not(test))]\nuse crate::Benign as Spawn;\n#[cfg(test)]\ninclude!(\"test_imports.rs\");\npub fn hidden() { let _ = self::Spawn::new(\"sh\"); }\npub struct Benign;\n",
    );
    write(
        &root,
        "src/test_imports.rs",
        "use std::process::Command as Spawn;\n",
    );
    write_executor(&root);
    write_lock(&root);

    let report = check(&root);
    assert_owned_call(&report, "process-spawn", "src/lib.rs");
    assert_no_owned_call(&report, "production-process", "src/lib.rs");
    reset(&root);
}
