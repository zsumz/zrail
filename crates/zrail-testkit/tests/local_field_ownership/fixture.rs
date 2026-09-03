//! Rafter-shaped fixture with nested modules and private type re-exports.

use std::{fs, path::PathBuf};

pub(super) fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zrail-local-field-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    reset(&root);
    for directory in ["src/node", "src/node/state"] {
        fs::create_dir_all(root.join(directory)).expect("create fixture source");
    }
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "zrail.toml", CONTRACT);
    write(&root, "src/lib.rs", LIBRARY);
    write(&root, "src/node.rs", NODE);
    write(&root, "src/node/state.rs", STATE);
    write(&root, "src/node/state/core.rs", CORE);
    write(&root, "src/node/state/derived.rs", DERIVED);
    write(&root, "src/node/construction.rs", CONSTRUCTION);
    write(&root, "src/node/dispatch.rs", DISPATCH);
    write(&root, "src/node/log.rs", LOG);
    write(&root, "src/node/lifecycle.rs", LIFECYCLE);
    write(&root, "src/node/heartbeat.rs", HEARTBEAT);
    root
}

fn write(root: &std::path::Path, path: &str, contents: &str) {
    fs::write(root.join(path), contents).expect("write fixture");
}

pub(super) fn reset(root: &PathBuf) {
    if root.exists() {
        fs::remove_dir_all(root).expect("reset fixture");
    }
}

const MANIFEST: &str = r#"[package]
name = "local-field-fixture"
version = "0.0.0"
edition = "2024"
"#;

const LIBRARY: &str = concat!("//! Local field ownership fixture.\n", "mod node;\n");

const NODE: &str = concat!(
    "//! Node owns protocol state and delegates transitions.\n",
    "mod construction;\n",
    "mod dispatch;\n",
    "mod heartbeat;\n",
    "mod lifecycle;\n",
    "mod log;\n",
    "mod state;\n",
    "use state::{DerivedState, VolatileState};\n",
    "pub struct LeaderState { pub ticks: u64 }\n",
    "pub struct Node {\n",
    "    pub volatile: VolatileState,\n",
    "    pub derived: DerivedState,\n",
    "    pub leader: LeaderState,\n",
    "}\n",
    "pub enum Input {\n",
    "    DangerousRawConfigurationProposal { configuration: Vec<u64> },\n",
    "}\n",
);

const STATE: &str = concat!(
    "//! State re-exports preserve the private declaring modules.\n",
    "mod core;\n",
    "mod derived;\n",
    "pub(super) use core::VolatileState;\n",
    "pub(super) use derived::DerivedState;\n",
);

const CORE: &str = r"//! Core state holds commit and apply indexes.
pub struct VolatileState { pub commit_index: u64, pub applied_index: u64 }
impl VolatileState {
    pub fn at_applied_index(index: u64) -> Self {
        Self { commit_index: index, applied_index: index }
    }
}
";

const DERIVED: &str = r"//! Derived state holds the configuration index.
pub struct DerivedState { pub configuration: Vec<u64> }
";

const CONSTRUCTION: &str = r"//! Construction owns the initial index values.
use super::state::{DerivedState, VolatileState};
use super::{LeaderState, Node};
impl Node {
    fn from_parts(index: u64) -> Result<Self, ()> {
        let mut volatile = VolatileState::at_applied_index(index);
        volatile.commit_index = index + 1;
        Ok(Self {
            volatile,
            derived: DerivedState { configuration: Vec::new() },
            leader: LeaderState { ticks: 0 },
        })
    }
    fn with_floor(index: u64) -> Result<Self, ()> {
        let mut node = Self::from_parts(index)?;
        node.volatile.applied_index = index;
        Ok(node)
    }
}
";

const DISPATCH: &str = r"//! Dispatch only reads unrelated input fields.
use super::Input;
fn dispatch(input: Input) {
    match input {
        Input::DangerousRawConfigurationProposal { configuration } => drop(configuration),
    }
}
";

const LOG: &str = r"//! Log transitions own configuration mutation.
use super::Node;
impl Node {
    fn clear_configuration(&mut self) { self.derived.configuration.clear(); }
}
";

const LIFECYCLE: &str = r"//! Lifecycle transitions replace leader state.
use super::{LeaderState, Node};
impl Node {
    fn reset_leader(&mut self) { self.leader = LeaderState { ticks: 0 }; }
}
";

const HEARTBEAT: &str = r"//! Heartbeats update interior leader bookkeeping.
use super::Node;
impl Node {
    fn tick(&mut self) { self.leader.ticks += 1; }
}
";

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
name = "commit-index-authority"
kind = "field-write"
within = ["src/**"]
match = "crate::node::state::core::VolatileState::commit_index"
allow = ["src/node/construction.rs"]
reason = "Construction establishes the commit index."

[[owner]]
name = "applied-index-authority"
kind = "field-write"
within = ["src/**"]
match = "crate::node::state::core::VolatileState::applied_index"
allow = ["src/node/construction.rs"]
reason = "Construction establishes the applied index."

[[owner]]
name = "configuration-authority"
kind = "field-mutation"
within = ["src/**"]
match = "crate::node::state::derived::DerivedState::configuration"
mutating_methods = ["clear"]
allow = ["src/node/log.rs"]
reason = "Configuration changes alongside the log."

[[owner]]
name = "leader-replacement-authority"
kind = "field-write"
within = ["src/**"]
match = "crate::node::Node::leader"
allow = ["src/node/lifecycle.rs"]
reason = "Only lifecycle transitions replace leader state."
"#;
