//! Narrow glob-import exceptions with exact syntax and reachability bounds.

use zrail_core::{AnalysisQuality, Finding, FindingSink, GlobImportMode};

use crate::inventory::FileClass;

use super::RuleContext;

pub(super) fn check_glob_imports(context: &RuleContext<'_>, findings: &mut FindingSink) {
    let mode = context.contract.source.rust.hygiene.glob_imports;
    if mode == GlobImportMode::Allow {
        return;
    }
    for file in &context.source.files {
        let effective = crate::source_policy::effective_file_role(
            &file.relative,
            file.class,
            &context.contract.source.rust,
        )
        .effective;
        for import in &file.glob_imports {
            if glob_import_is_allowed(mode, effective, file.reachability, import) {
                continue;
            }
            findings.push(
                Finding::error(
                    "RUST-HYG-009",
                    "rust.hygiene.glob-import",
                    "source-hygiene",
                    format!(
                        "{} source uses glob import {}::*",
                        crate::source_policy::role_name(effective),
                        import.target
                    ),
                )
                .at(&file.relative, Some(import.span))
                .with_analysis(AnalysisQuality::Exact)
                .with_help(
                    "name imported items explicitly or use a configured narrow glob-import mode",
                ),
            );
        }
    }
}

pub(crate) fn glob_import_is_allowed(
    mode: GlobImportMode,
    effective: FileClass,
    reachability: crate::source::Reachability,
    import: &crate::source::GlobImportFact,
) -> bool {
    import.guard == crate::source::SyntaxGuard::Never
        || mode == GlobImportMode::Allow
        || matches!(
            mode,
            GlobImportMode::FacadeReexportsOnly | GlobImportMode::FacadeReexportsAndTestSuper
        ) && effective == FileClass::Facade
            && import.lexical_scope.is_empty()
            && outward(&import.visibility)
        || mode == GlobImportMode::FacadeReexportsAndTestSuper
            && import.target == "super"
            && import.visibility == crate::source::BindingVisibility::Private
            && (reachability.is_test_only() || import.guard.is_test_only())
}

fn outward(visibility: &crate::source::BindingVisibility) -> bool {
    match visibility {
        crate::source::BindingVisibility::Public => true,
        crate::source::BindingVisibility::Restricted(path) => {
            path.first().is_some_and(|segment| segment != "self")
        }
        crate::source::BindingVisibility::Private => false,
    }
}
