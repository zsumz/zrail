//! Contextual `Self` retains its implementing type without crossing item scopes.

use super::fixture::{
    Repository, assert_complete, assert_no_owner, call_owner, capability_owner, exact_owner_count,
};

#[test]
fn self_associated_call_reaches_call_owner() {
    let repository = Repository::new(
        "self-call",
        &root_source("Self::spawn();"),
        CALL_OWNER_SOURCE,
        &call_owner("process-spawn", "crate::Process::spawn"),
    );
    assert_eq!(
        exact_owner_count(&repository.check(), "process-spawn", "src/lib.rs"),
        1
    );
}

#[test]
fn self_associated_function_reference_reaches_capability_owner() {
    let repository = Repository::new(
        "self-reference",
        &root_source("let _ = Self::spawn;"),
        CAPABILITY_OWNER_SOURCE,
        &capability_owner("process-spawn-capability", "crate::Process::spawn"),
    );
    assert_eq!(
        exact_owner_count(
            &repository.check(),
            "process-spawn-capability",
            "src/lib.rs",
        ),
        1
    );
}

#[test]
fn self_associated_const_reaches_capability_owner() {
    let repository = Repository::new(
        "self-const",
        "mod owner; pub struct Process; impl Process { pub const MAX: usize = 8; pub fn trespass() { let _ = Self::MAX; } }",
        "pub fn own() { let _ = crate::Process::MAX; }",
        &capability_owner("process-max-capability", "crate::Process::MAX"),
    );
    assert_eq!(
        exact_owner_count(&repository.check(), "process-max-capability", "src/lib.rs",),
        1
    );
}

#[test]
fn cross_file_impl_self_call_is_canonical() {
    let repository = Repository::new(
        "cross-file-self",
        "mod owner; mod process;",
        "pub fn own() { crate::process::Process::spawn(); }",
        &call_owner("process-spawn", "crate::process::Process::spawn"),
    );
    repository.write(
        "src/process.rs",
        "pub struct Process; impl Process { pub fn spawn() {} pub fn trespass() { Self::spawn(); } }",
    );
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "process-spawn", "src/process.rs"),
        1,
        "{}",
        report.human()
    );
}

#[test]
fn expression_include_inherits_current_self() {
    let repository = include_fixture("self-expression-include", "include!(\"expr.rs\");");
    repository.write("src/expr.rs", "{ Self::spawn(); }");
    assert_included_call(&repository, "src/expr.rs");
}

#[test]
fn nested_expression_include_inherits_current_self() {
    let repository = include_fixture("self-nested-include", "include!(\"outer.rs\");");
    repository.write("src/outer.rs", "{ include!(\"inner.rs\"); }");
    repository.write("src/inner.rs", "{ Self::spawn(); }");
    assert_included_call(&repository, "src/inner.rs");
}

#[test]
fn nested_item_does_not_inherit_invalid_self_context() {
    let repository = Repository::new(
        "self-nested-item",
        &root_source("include!(\"expr.rs\");"),
        CAPABILITY_OWNER_SOURCE,
        &capability_owner("process-spawn-capability", "crate::Process::spawn"),
    );
    repository.write(
        "src/expr.rs",
        "{ fn nested() { let _ = Self::spawn; } nested(); }",
    );
    assert_no_owner(
        &repository.check(),
        "process-spawn-capability",
        "src/expr.rs",
    );
}

fn include_fixture(name: &str, body: &str) -> Repository {
    Repository::new(
        name,
        &root_source(body),
        CALL_OWNER_SOURCE,
        &call_owner("process-spawn", "crate::Process::spawn"),
    )
}

fn assert_included_call(repository: &Repository, path: &str) {
    let report = repository.check();
    assert_complete(&report);
    assert_eq!(
        exact_owner_count(&report, "process-spawn", path),
        1,
        "{}",
        report.human()
    );
}

fn root_source(body: &str) -> String {
    format!(
        "mod owner; pub struct Process; impl Process {{ pub fn spawn() {{}} pub fn trespass() {{ {body} }} }}"
    )
}

const CALL_OWNER_SOURCE: &str = "pub fn own() { crate::Process::spawn(); }";
const CAPABILITY_OWNER_SOURCE: &str = "pub fn own() { let _ = crate::Process::spawn; }";
