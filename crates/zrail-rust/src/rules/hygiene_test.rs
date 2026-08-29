//! Glob hygiene admits only exact outward facade re-exports.

use zrail_core::{GlobImportMode, SourceSpan};

use crate::{
    inventory::FileClass,
    source::{BindingVisibility, GlobImportFact, Reachability, ReachabilityKind, SyntaxGuard},
};

use super::glob_import_is_allowed;

#[test]
fn facade_mode_requires_outward_top_level_reexports() {
    let public = fact(BindingVisibility::Public, SyntaxGuard::Ordinary, false);
    let crate_visible = fact(
        BindingVisibility::Restricted(vec!["crate".into()]),
        SyntaxGuard::Ordinary,
        false,
    );
    let self_visible = fact(
        BindingVisibility::Restricted(vec!["self".into()]),
        SyntaxGuard::Ordinary,
        false,
    );
    let private = fact(BindingVisibility::Private, SyntaxGuard::Ordinary, false);
    let scoped = fact(BindingVisibility::Public, SyntaxGuard::Ordinary, true);

    assert!(allowed(FileClass::Facade, &public));
    assert!(allowed(FileClass::Facade, &crate_visible));
    assert!(!allowed(FileClass::Facade, &self_visible));
    assert!(!allowed(FileClass::Facade, &private));
    assert!(!allowed(FileClass::Facade, &scoped));
    assert!(!allowed(FileClass::Implementation, &public));
}

#[test]
fn syntactically_absent_globs_do_not_violate_even_deny_mode() {
    let absent = fact(BindingVisibility::Private, SyntaxGuard::Never, false);

    assert!(glob_import_is_allowed(
        GlobImportMode::Deny,
        FileClass::Implementation,
        Reachability::from_kind(ReachabilityKind::Production),
        &absent,
    ));
}

#[test]
fn test_super_mode_requires_exact_private_test_only_import() {
    let mut import = fact(BindingVisibility::Private, SyntaxGuard::Ordinary, false);
    import.target = "super".into();
    let production = Reachability::from_kind(ReachabilityKind::Production);
    let test = Reachability::from_kind(ReachabilityKind::Test);

    assert!(glob_import_is_allowed(
        GlobImportMode::FacadeReexportsAndTestSuper,
        FileClass::Test,
        test,
        &import,
    ));
    assert!(!glob_import_is_allowed(
        GlobImportMode::FacadeReexportsAndTestSuper,
        FileClass::Implementation,
        production,
        &import,
    ));

    import.guard = SyntaxGuard::TestOnly;
    assert!(glob_import_is_allowed(
        GlobImportMode::FacadeReexportsAndTestSuper,
        FileClass::Implementation,
        production,
        &import,
    ));

    import.target = "crate::support".into();
    assert!(!glob_import_is_allowed(
        GlobImportMode::FacadeReexportsAndTestSuper,
        FileClass::Test,
        test,
        &import,
    ));
    import.target = "super".into();
    import.visibility = BindingVisibility::Public;
    assert!(!glob_import_is_allowed(
        GlobImportMode::FacadeReexportsAndTestSuper,
        FileClass::Test,
        test,
        &import,
    ));
}

fn allowed(class: FileClass, fact: &GlobImportFact) -> bool {
    glob_import_is_allowed(
        GlobImportMode::FacadeReexportsOnly,
        class,
        Reachability::from_kind(ReachabilityKind::Production),
        fact,
    )
}

fn fact(visibility: BindingVisibility, guard: SyntaxGuard, scoped: bool) -> GlobImportFact {
    GlobImportFact {
        target: "crate::api".into(),
        visibility,
        span: span(),
        guard,
        lexical_scope: scoped.then_some(span()).into_iter().collect(),
    }
}

const fn span() -> SourceSpan {
    SourceSpan {
        line: 1,
        column: 0,
        end_line: 1,
        end_column: 1,
    }
}
