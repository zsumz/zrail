//! A foreign block was implicitly unsafe before Rust 2024.

extern "C" {
    fn legacy();
}
