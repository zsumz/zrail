//! Dual-reachable test fixture.

#[cfg(test)]
#[path = "worker_test.rs"]
mod worker_test;

#[path = "worker_test.rs"]
mod production_worker_test;
