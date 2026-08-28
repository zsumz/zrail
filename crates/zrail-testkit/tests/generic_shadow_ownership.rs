//! Generic parameters never borrow constructor identity across Rust namespaces.

#[path = "generic_shadow_ownership/direct.rs"]
mod direct;
#[path = "generic_shadow_ownership/fields.rs"]
mod fields;
#[path = "generic_shadow_ownership/fixture.rs"]
mod fixture;
#[path = "generic_shadow_ownership/includes.rs"]
mod includes;
#[path = "generic_shadow_ownership/observed.rs"]
mod observed;
