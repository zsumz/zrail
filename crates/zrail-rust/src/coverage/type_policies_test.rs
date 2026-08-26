//! Type coverage exposes authored policy and actual source observations.

use zrail_core::AnalysisQuality;

use super::super::GovernedSurfaceReport;

pub(crate) fn assert_type_policy_coverage(report: &GovernedSurfaceReport) {
    assert_eq!(report.schema, 5);
    let import_policy = report
        .source_policies
        .iter()
        .find(|policy| policy.policy_id == "rust:duplication:import:clone")
        .expect("duplication import policy");
    assert_eq!(import_policy.occurrences.len(), 1);
    assert_eq!(import_policy.occurrences[0].operation, "import");
    assert_eq!(import_policy.occurrences[0].observed, "clone");
    assert!(!import_policy.occurrences[0].allowed);

    let type_policy = report
        .type_policies
        .iter()
        .find(|policy| policy.name == "record-shape")
        .expect("exact type policy");
    assert_eq!(type_policy.identity, "crate::Record");
    assert_eq!(type_policy.visibility.as_deref(), Some("pub"));
    assert_eq!(
        type_policy.fields.as_ref().expect("expected fields")[0].type_identity,
        "usize"
    );
    let declaration = type_policy
        .observations
        .iter()
        .find(|observation| observation.operation == "declaration")
        .expect("declaration observation");
    assert!(declaration.allowed);
    assert_eq!(declaration.quality, AnalysisQuality::Exact);
    assert_eq!(
        declaration.fields.as_ref().expect("observed fields")[0].type_identity,
        "usize"
    );
    let derive = type_policy
        .observations
        .iter()
        .find(|observation| observation.operation == "derive")
        .expect("guarded derive observation");
    assert_eq!(derive.guard, "cfg:feature=\"copyable\"");
    assert!(derive.allowed);
    assert!(
        derive
            .compilation_domains
            .iter()
            .all(|domain| domain.feature_world.is_none())
    );
    let implementation = type_policy
        .observations
        .iter()
        .find(|observation| observation.operation == "manual-impl")
        .expect("unresolved manual impl observation");
    assert!(!implementation.allowed);
    assert_eq!(implementation.quality, AnalysisQuality::Unresolved);
    assert!(implementation.observed.contains("Clone for"));
    assert!(
        implementation
            .compilation_domains
            .iter()
            .any(|domain| domain.package == "audit-app" && domain.mode == "library")
    );
}
