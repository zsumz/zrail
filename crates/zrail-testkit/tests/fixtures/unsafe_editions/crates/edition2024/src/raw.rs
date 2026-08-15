//! Explicit Rust 2024 unsafe boundaries.

#[unsafe(no_mangle)]
pub extern "C" fn exposed() {}

unsafe extern "C" {
    safe fn modern();
}
