//! Adversarial authored expression contexts for the exact frozen collector.

pub(super) const SUBJECT: &str = "kind='written-expression-paths',suffixes=['ConnectionSet::new','ConnectionSet::turn_component','ConnectionSet::poll_io','ConnectionSet::wake_handle','ConnectionSet::pulse_handle','DirectSet::new','DirectSet::turn_component','DirectSet::poll_io','DirectSet::wake_handle','DirectSet::pulse_handle','Source::register','Source::reregister','Source::deregister']";

pub(super) const CASES: [(&str, usize); 22] = [
    (
        "fn run() { ConnectionSet::new(); let _f = ConnectionSet::new; }",
        2,
    ),
    (
        "fn run() { ::external::ConnectionSet::<T>::new::<U>(); }",
        1,
    ),
    ("fn run() { <T as transport::Source>::register(); }", 1),
    ("fn run() { <ConnectionSet>::new(); }", 0),
    (
        "use transport::ConnectionSet as Alias; fn run() { Alias::new(); }",
        0,
    ),
    (
        "use transport::{ConnectionSet as Alias, Source, *}; type T = ConnectionSet;",
        0,
    ),
    ("fn run() { receiver.new(); }", 0),
    ("#[doc = ConnectionSet::new] struct State;", 1),
    ("#[check(ConnectionSet::new)] struct State;", 0),
    ("type X = Array<{ ConnectionSet::new }>;", 1),
    ("struct State<T = Array<{ Source::register }>>(T);", 1),
    (
        "const X: usize = ConnectionSet::new; static Y: usize = Source::register;",
        2,
    ),
    ("enum X { A = ConnectionSet::new }", 1),
    (
        "trait X { fn run() { Source::register(); } const X: usize = DirectSet::new; }",
        2,
    ),
    ("impl X { const X: usize = DirectSet::new; }", 1),
    (
        "fn run() { fn nested() { ConnectionSet::new(); } let _f = || Source::register(); }",
        2,
    ),
    (
        "#[cfg(any())] fn a() { Source::register(); } #[cfg(test)] fn b() { Source::register(); } #[cfg(not(test))] fn c() { Source::register(); }",
        3,
    ),
    (
        "macro_rules! hidden { () => { ConnectionSet::new() }; } fn run() { stringify!(Source::register()); }",
        0,
    ),
    (
        "fn run() { let _s = \"Source::register()\"; /* ConnectionSet::new() */ }",
        0,
    ),
    (
        "fn run() { r#ConnectionSet::new(); ConnectionSet::r#new(); }",
        0,
    ),
    (
        "fn run() { ConnectionSet::new::<{ DirectSet::poll_io }>(); }",
        2,
    ),
    (
        "fn run(x: X) { match x { ConnectionSet::new => (), _ => () } }",
        1,
    ),
];
