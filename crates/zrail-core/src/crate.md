
`zrail-core` is the policy engine shared by the zrail command-line tools. It
deliberately does not walk Rust syntax or invoke external programs. Callers load
a repository-bounded `zrail.toml` contract, supply observed lock state, and
compare two accepted architecture states.

# Primary flow

[`load_contract`] parses the root contract and its repository-local imports,
validates the merged schema, and returns typed policy plus a digest of the exact
source bytes. [`LockFile`] records observations tied to that digest.
[`compare_architecture_checked`] then classifies policy and lock changes while
treating absent, stale, or unsupported lock authority as
[`ChangeKind::Unknown`]. [`DiffReport::denies_grants`] is the fail-closed
decision for automation that must reject grants, debt, and unknown changes.

```
use std::{fs, path::Path, time::{SystemTime, UNIX_EPOCH}};
use zrail_core::{LockFile, compare_architecture_checked, load_contract};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
let root = std::env::temp_dir().join(format!(
    "zrail-core-doc-{}-{nonce}",
    std::process::id()
));
fs::create_dir(&root)?;
fs::write(root.join("zrail.toml"), r#"
schema = 1
adapters = ["rust"]

[repository]
roots = ["crates"]
exclude = []
workspace_members = "exact"
nested_git = "deny"
submodules = "deny"
symlinks = "inside"

[dependencies]
mode = "observed"
unassigned_packages = "allow"
cycles = "allow"

[source.rust]
module_docs = "allow"
facades = "allow"
tests = "allow"

[source.rust.hygiene]
unsafe = "allow"
lint_suppressions = "allow"
"#)?;

let accepted = load_contract(&root, Path::new("zrail.toml"))?;
let lock = LockFile::new(&accepted.sha256);
let diff = compare_architecture_checked(
    &accepted.contract,
    &accepted.sha256,
    Some(&lock),
    &accepted.contract,
    &accepted.sha256,
    Some(&lock),
);
assert!(!diff.denies_grants());
fs::remove_dir_all(root)?;
# Ok(())
# }
```

# Determinism and errors

Contract imports, findings, lock entries, and diff changes are normalized into
deterministic order before their public serialized forms are emitted. Parsing
and filesystem helpers reject symlinks, non-regular files, repository escapes,
unknown contract keys, and configured safety-limit violations. Error strings
are intended for people and diagnostics; they are not a versioned
machine-readable protocol.
