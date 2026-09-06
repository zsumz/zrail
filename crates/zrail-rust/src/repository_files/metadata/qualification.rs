//! Trusted metadata qualification reuses the stock file analyzer and frozen assertion body.

#[path = "fixtures.rs"]
mod fixtures;
#[path = "legacy.rs"]
mod legacy;
#[path = "model.rs"]
mod model;
#[path = "mutations.rs"]
mod mutations;

#[cfg(test)]
#[path = "qualification_test.rs"]
mod qualification_test;
