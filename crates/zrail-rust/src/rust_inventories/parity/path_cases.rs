//! Every frozen path context preserves written owner membership without resolving aliases.

pub(super) const SUBJECT: &str = "kind='written-paths-containing',names=['ConnectionSet']";

pub(super) const CASES: [(&str, usize); 26] = [
    ("fn f(x: ConnectionSet) -> ConnectionSet { x }", 2),
    (
        "fn f() { ConnectionSet::new(); let _f = ConnectionSet::new; }",
        2,
    ),
    ("fn f() { ::external::ConnectionSet::<T>::new(); }", 1),
    ("fn f() { ConnectionSet::ConnectionSet::new(); }", 1),
    ("fn f() { <ConnectionSet>::new(); }", 1),
    (
        "type X = <ConnectionSet as Trait<ConnectionSet>>::Output;",
        2,
    ),
    ("fn f() { ConnectionSet::<ConnectionSet>::new(); }", 2),
    ("use external::ConnectionSet as Alias; fn f(x: Alias) {}", 0),
    (
        "use external::{ConnectionSet, ConnectionSet as Alias, *};",
        0,
    ),
    ("pub(in crate::ConnectionSet) use external::X;", 1),
    ("pub(in crate::ConnectionSet) mod inner {}", 1),
    ("#[ConnectionSet] struct X;", 1),
    ("#[ConnectionSet::checked] struct X;", 1),
    ("#[checked(ConnectionSet)] struct X;", 0),
    ("#[doc = ConnectionSet::DOC] struct X;", 1),
    (
        "struct X(ConnectionSet); enum Y { X { value: ConnectionSet } }",
        2,
    ),
    ("trait X: ConnectionSet {} impl ConnectionSet for X {}", 2),
    (
        "struct ConnectionSet; mod ConnectionSet {} fn ConnectionSet() {}",
        0,
    ),
    ("fn f(x: X) { match x { ConnectionSet => (), _ => () } }", 0),
    (
        "fn f(x: X) { match x { ConnectionSet::X => (), ConnectionSet{} => (), ConnectionSet(_) => () } }",
        3,
    ),
    ("ConnectionSet! { ConnectionSet::new(); }", 1),
    (
        "macro_rules! ConnectionSet { () => { ConnectionSet::new() }; }",
        0,
    ),
    (
        "fn f() { let _s = \"ConnectionSet\"; /* ConnectionSet */ stringify!(ConnectionSet); }",
        0,
    ),
    ("fn f(x: r#ConnectionSet, y: connectionSet) {}", 0),
    (
        "#[cfg(test)] fn a(x: ConnectionSet) {} #[cfg(not(test))] fn b(x: ConnectionSet) {} #[cfg(any())] fn c(x: ConnectionSet) {}",
        3,
    ),
    (
        "fn f() { fn nested(x: ConnectionSet) {} let _f = |x: ConnectionSet| x; }",
        2,
    ),
];
