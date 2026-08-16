//! Current locks distinguish exact and deliberately unresolved Rust crate roots.

use zrail_core::{
    LockFile, LockedDependency, LockedDependencyKind, LockedDependencyScope,
    LockedDependencySource, LockedPackage,
};

#[test]
fn current_semantics_record_an_unresolved_crate_root_by_omission() {
    let lock = lock_with(dependency(None));

    let rendered = lock.render().expect("unresolved root remains explicit");

    assert!(!rendered.contains("crate_root"));
    assert_eq!(lock.packages[0].dependencies[0].crate_root, None);
}

#[test]
fn current_semantics_reject_an_invalid_effective_crate_root() {
    for root in ["unresolved-root", "r#async", "self"] {
        let lock = lock_with(dependency(Some(root)));
        let error = lock
            .render()
            .expect_err("invalid crate-root identity must fail");
        assert!(error.to_string().contains("invalid effective crate root"));
    }
}

#[test]
fn current_semantics_require_internal_crate_roots() {
    let mut dependency = dependency(None);
    dependency.scope = LockedDependencyScope::Internal;
    dependency.source = Some(LockedDependencySource::WorkspaceMember {
        directory: "crates/core".into(),
        requirement: None,
    });
    let lock = lock_with(dependency);

    let error = lock.render().expect_err("internal root must be exact");

    assert!(
        error
            .to_string()
            .contains("requires an effective crate root")
    );
}

fn lock_with(dependency: LockedDependency) -> LockFile {
    let mut lock = LockFile::new("0".repeat(64));
    lock.packages.push(LockedPackage {
        name: "app".into(),
        dependencies: vec![dependency],
    });
    lock
}

fn dependency(crate_root: Option<&str>) -> LockedDependency {
    LockedDependency {
        alias: Some("core".into()),
        name: "core".into(),
        crate_root: crate_root.map(str::to_owned),
        kind: LockedDependencyKind::Normal,
        scope: LockedDependencyScope::External,
        target: None,
        optional: Some(false),
        default_features: Some(true),
        features: Vec::new(),
        source: Some(LockedDependencySource::Registry {
            registry: None,
            index: None,
            requirement: "1".into(),
        }),
    }
}
