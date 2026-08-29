//! Bound subjects retain qualified and unqualified projection structure.

use std::collections::BTreeSet;

use syn::Type;

use super::{AssociatedSegment, BoundSubject, GenericPathIdentity, ProjectionIdentity};
use crate::source::GenericArgumentsIdentity;

#[test]
fn qualified_projection_retains_subject_and_qualifier() {
    let ty = syn::parse_str::<Type>("<T as api::Provider>::Factory").expect("qualified type");
    let subject =
        BoundSubject::from_type(&ty, &BTreeSet::from(["T".into()])).expect("bound subject");
    assert_eq!(
        subject,
        BoundSubject::Projection {
            root: "T".into(),
            projection: ProjectionIdentity {
                qualifying_trait: Some(GenericPathIdentity {
                    path: "api::Provider".into(),
                    arguments: GenericArgumentsIdentity::Exact(Vec::new()),
                }),
                associated: vec![AssociatedSegment {
                    name: "Factory".into(),
                    arguments: GenericArgumentsIdentity::Exact(Vec::new()),
                }],
            },
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
            projection: ProjectionIdentity {
                qualifying_trait: None,
                associated: vec![AssociatedSegment {
                    name: "Factory".into(),
                    arguments: GenericArgumentsIdentity::Exact(Vec::new()),
                }],
            },
        }
    );
}

#[test]
fn qualifier_and_associated_generic_arguments_are_distinct() {
    let left = syn::parse_str::<Type>("<T as Provider<A>>::Item<X>").expect("left type");
    let right = syn::parse_str::<Type>("<T as Provider<B>>::Item<X>").expect("right type");
    let declared = BTreeSet::from(["T".into()]);

    let left = BoundSubject::from_type(&left, &declared).expect("left subject");
    let right = BoundSubject::from_type(&right, &declared).expect("right subject");

    assert_ne!(left, right);
}
