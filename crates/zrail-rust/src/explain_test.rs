//! Effective path-policy explanations match rule evaluation.

use zrail_core::{FacadeMode, ModuleDocsMode};

use crate::inventory::FileClass;

use super::policy::{declarative_shape, module_docs_required};

#[test]
fn facade_and_entrypoint_modes_are_reported_independently() {
    assert_eq!(
        declarative_shape(
            FileClass::Facade,
            FacadeMode::Declarative,
            FacadeMode::Allow
        ),
        Some(true)
    );
    assert_eq!(
        declarative_shape(
            FileClass::EntryPoint,
            FacadeMode::Declarative,
            FacadeMode::Allow
        ),
        Some(false)
    );
    assert_eq!(
        declarative_shape(
            FileClass::Generated,
            FacadeMode::Declarative,
            FacadeMode::Allow
        ),
        None
    );
}

#[test]
fn generated_source_reports_its_effective_module_doc_exemption() {
    assert!(!module_docs_required(
        FileClass::Generated,
        ModuleDocsMode::Required
    ));
    assert!(module_docs_required(
        FileClass::Implementation,
        ModuleDocsMode::Required
    ));
}
