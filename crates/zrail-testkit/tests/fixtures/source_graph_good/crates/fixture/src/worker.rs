//! Source graph fixture implementation.

mod nested;

mod platform {
    #[path = "tls.rs"]
    mod local;
}

include!("included.rs");

pub(crate) fn included_value() -> usize {
    include!("value.rs")
}
