//! Bound subjects retain qualified and unqualified projection structure.

use std::collections::BTreeSet;

use syn::Type;

use super::BoundSubject;

#[test]
fn qualified_projection_retains_subject_and_qualifier() {
    let ty = syn::parse_str::<Type>("<T as api::Provider>::Factory").expect("qualified type");
    let subject =
        BoundSubject::from_type(&ty, &BTreeSet::from(["T".into()])).expect("bound subject");
    assert_eq!(
        subject,
        BoundSubject::Projection {
            root: "T".into(),
            qualifying_trait: Some("api::Provider".into()),
            associated: vec!["Factory".into()],
        }
    );
}

#[test]
fn self_projection_is_a_typed_subject() {
    let ty = syn::parse_str::<Type>("Self::Factory").expect("self projection");
    let subject =
        BoundSubject::from_type(&ty, &BTreeSet::from(["Self".into()])).expect("bound subject");
    assert_eq!(
        subject,
        BoundSubject::Projection {
            root: "Self".into(),
            qualifying_trait: None,
            associated: vec!["Factory".into()],
        }
    );
}
