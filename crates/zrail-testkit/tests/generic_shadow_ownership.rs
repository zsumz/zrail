//! Generic parameters never borrow constructor identity across Rust namespaces.

#[path = "generic_shadow_ownership/aliased_impl_self.rs"]
mod aliased_impl_self;
#[path = "generic_shadow_ownership/associated_includes.rs"]
mod associated_includes;
#[path = "generic_shadow_ownership/associated_substitutions.rs"]
mod associated_substitutions;
#[path = "generic_shadow_ownership/bound_subjects.rs"]
mod bound_subjects;
#[path = "generic_shadow_ownership/direct.rs"]
mod direct;
#[path = "generic_shadow_ownership/fields.rs"]
mod fields;
#[path = "generic_shadow_ownership/fixture.rs"]
mod fixture;
#[path = "generic_shadow_ownership/fragment_syntax.rs"]
mod fragment_syntax;
#[path = "generic_shadow_ownership/fragment_trait_scope.rs"]
mod fragment_trait_scope;
#[path = "generic_shadow_ownership/includes.rs"]
mod includes;
#[path = "generic_shadow_ownership/observed.rs"]
mod observed;
#[path = "generic_shadow_ownership/qualified_projections.rs"]
mod qualified_projections;
#[path = "generic_shadow_ownership/self_identity.rs"]
mod self_identity;
#[path = "generic_shadow_ownership/self_trait.rs"]
mod self_trait;
#[path = "generic_shadow_ownership/trait_bounds.rs"]
mod trait_bounds;
