//! Frozen explicit-renaming semantics include same-name and underscore destinations.

pub(super) const SUBJECT: &str = "kind='written-import-renames',names=['ConnectionSet','DirectSet','RegisteredTransport','SlotTransport','Source']";

pub(super) const CASES: &[(&str, &[(&str, usize)])] = &[
    ("use p::Source as Io;", &[("Source as Io", 1)]),
    ("use p::Source as Source;", &[("Source as Source", 1)]),
    ("use p::Source as _;", &[("Source as _", 1)]),
    (
        "use p::{Source as Io, nested::{Source as Other, ConnectionSet as Set}, *};",
        &[
            ("Source as Io", 1),
            ("Source as Other", 1),
            ("ConnectionSet as Set", 1),
        ],
    ),
    (
        "use p::Source as Io; use q::Source as Io;",
        &[("Source as Io", 2)],
    ),
    (
        "use ::external::ConnectionSet as Set;",
        &[("ConnectionSet as Set", 1)],
    ),
    ("pub use p::Source as Io;", &[("Source as Io", 1)]),
    (
        "pub(in crate::inner) use p::Source as Io;",
        &[("Source as Io", 1)],
    ),
    ("use p::Source; use p::{Source, *};", &[]),
    ("use p::Other as Source;", &[]),
    ("use p::r#Source as Io;", &[]),
    ("use p::Source as r#Io;", &[("Source as r#Io", 1)]),
    ("use p::source as Io;", &[]),
    ("use p::Source::{self as Alias};", &[]),
    ("extern crate Source as Io;", &[]),
    ("type Source = Io; fn f(Source: Io) {}", &[]),
    (
        "fn f() { use p::Source as Io; } mod nested { use p::Source as Io; }",
        &[("Source as Io", 2)],
    ),
    (
        "#[cfg(test)] use p::Source as Io; #[cfg(not(test))] use p::Source as Io; #[cfg(any())] use p::Source as Io;",
        &[("Source as Io", 3)],
    ),
    (
        "#[doc = { use p::Source as Io; \"\" }] struct X;",
        &[("Source as Io", 1)],
    ),
    (
        "struct X([u8; { use p::Source as Io; 0 }]);",
        &[("Source as Io", 1)],
    ),
    (
        "macro_rules! hidden { () => { use p::Source as Io; }; }",
        &[],
    ),
    (
        "fn f() { stringify!(use p::Source as Io;); let _s = \"use p::Source as Io;\"; /* use p::Source as Io; */ }",
        &[],
    ),
];
