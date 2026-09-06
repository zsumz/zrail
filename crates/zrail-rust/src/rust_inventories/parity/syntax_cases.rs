//! Authored syntax contexts exercise both frozen and native collectors.

pub(super) const CASES: [&str; 20] = [
    "fn run(x: X) { x.poll_io(); }",
    "fn run(x: X) { x.poll_io(); x.poll_io(); }",
    "fn run(x: X) { let f = X::poll_io; X::poll_io(x); }",
    "fn run(x: X) { let _s = \"x.poll_io()\"; /* x.poll_io(); */ }",
    "fn run(x: X) { opaque!(x.poll_io()); stringify!(x.poll_io()); }",
    "macro_rules! hidden { () => { x.poll_io(); } }",
    "fn run(x: X) { #[cfg(any())] x.poll_io(); #[cfg(test)] x.poll_io(); }",
    "fn run(x: X) { let _f = || x.poll_io(); fn nested(x: X) { x.poll_io(); } }",
    "fn run(x: X) { let _ = x.poll_io::<{ x.wake_handle() }>(); }",
    "fn run() { let _ = X::<{ x.poll_io() }>::value; }",
    "fn run() { let _ = <X<{ x.poll_io() }> as Trait>::VALUE; }",
    "fn run() { let _ = <X as Trait<{ x.poll_io() }>>::VALUE; }",
    "type Buffer = [u8; x.poll_io()];",
    "enum Tag { First = x.poll_io() }",
    "#[doc = x.poll_io()] fn run() {}",
    "fn run(#[attribute = x.poll_io()] x: X) {}",
    "fn run(x: X) { x.r#poll_io(); x.Poll_io(); x.poll_io(); }",
    "use crate::{Thing as Alias, nested::*}; fn run(x: Alias) { x.poll_io(); }",
    "impl Trait for X { fn run(&self) { self.poll_io(); } }",
    "trait Trait { const C: usize = x.poll_io(); fn run(&self) { self.poll_io(); } }",
];
