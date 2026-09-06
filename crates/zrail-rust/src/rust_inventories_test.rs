//! Frozen collector parity checks syntax corners before downstream replacement is claimed.

#[path = "rust_inventories/parity/legacy.rs"]
mod legacy;
#[path = "rust_inventories/parity/native.rs"]
mod native;

#[test]
fn frozen_transport_methods_match_every_parsed_expression_context() {
    assert_eq!(legacy::expected().len(), 11);
    assert_eq!(legacy::expected().values().sum::<usize>(), 20);
    let cases = [
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
    let root = std::env::temp_dir().join(format!(
        "zrail-method-parity-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    for (index, source) in cases.into_iter().enumerate() {
        let expected = legacy::observed("src/sample.rs", source);
        let observed = native::observed(&root, source);
        assert_eq!(observed, expected, "case {index}: {source}");
    }
}
