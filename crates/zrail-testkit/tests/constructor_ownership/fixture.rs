//! Disposable repository exercising constructor ownership through the public API.

use std::{fs, path::PathBuf};

pub(super) struct Repository {
    root: PathBuf,
}

impl Repository {
    pub(super) fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zrail-constructor-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset constructor fixture");
        }
        fs::create_dir_all(root.join("src")).expect("create constructor source");
        for (path, contents) in FILES {
            fs::write(root.join(path), contents).expect("write constructor fixture");
        }
        Self { root }
    }

    pub(super) fn path(&self) -> &std::path::Path {
        &self.root
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).expect("remove constructor fixture");
        }
    }
}

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"constructor-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    ),
    ("zrail.toml", CONTRACT),
    (
        "src/lib.rs",
        "//! Constructor fixture.\npub mod model;\nmod owner;\nmod self_trespass;\nmod trespasser;\nmod values;\n",
    ),
    (
        "src/model.rs",
        r"//! Constructor declarations.
#![allow(non_camel_case_types)]
pub struct ticket(pub u64);
pub struct marker;
pub enum choice { ready(u64), idle, record { value: u64 } }
impl choice { pub fn ticket(_: u64) {} }
pub struct Ticket(pub u64);
pub struct State { pub epoch: u64, pub secret: u64 }
",
    ),
    (
        "src/owner.rs",
        r"//! Declared constructor owner.
use crate::model::{Ticket, State, choice, marker, ticket};
fn own() {
    let _ = ticket(0);
    let _ = marker;
    let _ = choice::ready(0);
    let _ = choice::idle;
    let _ = choice::record { value: 0 };
    let _ = Ticket(0);
    let _ = State { epoch: 0, secret: 0 };
}
fn inspect(state: &State) { let _ = state.secret; }
",
    ),
    (
        "src/trespasser.rs",
        r"//! Constructor trespassers.
use crate::model::ticket as make;
use crate::model::ticket as r#match;
use crate::model::marker as value;
use crate::model::choice::ready as variant;
use crate::model::choice::*;
fn trespass() {
    let _ = crate::model::ticket(1);
    let _ = make(2);
    let _ = r#match(3);
    let _ = value;
    let _ = variant(4);
    let _ = ready(5);
    let _ = idle;
    let _ = record { value: 6 };
    let _ = crate::model::Ticket(7);
    let _ = crate::model::State { epoch: 8, secret: 9 };
}
",
    ),
    (
        "src/self_trespass.rs",
        r"//! Cross-file Self trespassers.
impl crate::model::ticket { fn mint() -> Self { Self(1) } }
impl crate::model::marker { fn mint() -> Self { Self } }
impl crate::model::choice {
    fn tuple() -> Self { Self::ready(1) }
    fn unit() -> Self { Self::idle }
    fn record() -> Self { Self::record { value: 1 } }
}
impl crate::model::Ticket { fn mint() -> Self { Self(1) } }
impl crate::model::State {
    fn mint() -> Self { Self { epoch: 1, secret: 2 } }
    fn update(previous: Self) -> Self { Self { epoch: 3, ..previous } }
}
",
    ),
    (
        "src/values.rs",
        r"//! Proven non-constructor values.
fn ticket(_: u64) {}
const MARKER: u64 = 1;
fn values(ticket: fn(u64), marker: u64) {
    ticket(1);
    let _ = marker;
    let _ = MARKER;
    crate::model::choice::ticket(2);
    let _ = crate::model::ticket;
    let _ = crate::model::marker();
}
",
    ),
];

const CONTRACT: &str = r#"schema = 1
adapters = ["rust"]
[repository]
roots = ["."]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"
[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "deny"
[source.rust]
module_docs = "required"
facades = "allow"
tests = "allow"
[source.rust.macros]
mode = "allow"
[source.rust.hygiene]
unsafe = "deny"
lint_suppressions = "allow"

[[owner]]
name = "ticket-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::ticket"
allow = ["src/owner.rs"]
reason = "ticket construction stays centralized."

[[owner]]
name = "marker-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::marker"
allow = ["src/owner.rs"]
reason = "marker construction stays centralized."

[[owner]]
name = "ready-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::choice::ready"
allow = ["src/owner.rs"]
reason = "ready construction stays centralized."

[[owner]]
name = "idle-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::choice::idle"
allow = ["src/owner.rs"]
reason = "idle construction stays centralized."

[[owner]]
name = "record-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::choice::record"
allow = ["src/owner.rs"]
reason = "record construction stays centralized."

[[owner]]
name = "uppercase-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::Ticket"
allow = ["src/owner.rs"]
reason = "Ticket construction stays centralized."

[[owner]]
name = "state-construction"
kind = "type-construction"
within = ["src/**"]
match = "crate::model::State"
allow = ["src/owner.rs"]
reason = "State construction stays centralized."

[[owner]]
name = "state-secret"
kind = "field-read"
within = ["src/**"]
match = "crate::model::State::secret"
allow = ["src/owner.rs"]
reason = "State secret reads stay centralized."
"#;
