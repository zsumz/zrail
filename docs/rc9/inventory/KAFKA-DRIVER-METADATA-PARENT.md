# Fixed metadata manifest parents

`KD-METADATA-PARENT` binds `tests/guardrails/release_metadata.rs` at frozen
kafka-driver `a45be8071e6a663cd4ae3142cc5304ece9fdf45e`. The complete original
metadata body and its `PUBLIC_PACKAGES` constant are reused unchanged from the
existing metadata extraction registry; only the trusted workspace-root closure
selects the isolated input tree.

## Precondition proof

Every registry path is a nonempty, canonical relative path ending in the normal
filename `Cargo.toml`. Joining any absolute workspace root therefore yields a
path with that final filename, not a filesystem root or empty path. Rust
`Path::parent` returns its lexical prefix. Appending `LICENSE` produces exactly
these already translated policy inputs:

| Manifest | Parent relative to root | License policy input |
| --- | --- | --- |
| `Cargo.toml` | empty | `LICENSE` |
| `crates/kafka-driver-core/Cargo.toml` | `crates/kafka-driver-core` | `crates/kafka-driver-core/LICENSE` |
| `crates/kafka-driver-transport/Cargo.toml` | `crates/kafka-driver-transport` | `crates/kafka-driver-transport/LICENSE` |

The corresponding `kd-metadata-<package>-license-copy` policies already require
UTF-8 byte equality against `LICENSE`. Their names, exact includes, empty
exclusions, file entry mode, reference and UTF-8 requirement are checked without
editing the fragment. No new root contract or license authority is introduced.

This branch cannot fail for the frozen registry. Empty and filesystem-root
paths have no parent, but neither is a possible joined manifest input here.
Those negative controls are not fabricated failures of the original assertion.
Missing/unreadable/unequal licenses remain separate read/content failures.

## Trusted qualification scope

Four absolute anchor shapes exercise twelve mappings: filesystem root, checkout,
nested Unicode/spaces and an uncreated lexical descendant. Synthetic anchors
are not filesystem or permission evidence. Seven altered policy mappings are
rejected, including same-count wrong paths, changed reference and removed UTF-8
requirements. These mutation tests do not grant or accept changed authority.

The original full metadata body runs against all eleven frozen inputs, while
stock native analysis binds the three existing license policies and their exact
physical input hashes. Successful completion demonstrates the original parent
branch was reached for each of the three public packages. The runner requires
five exact tests, including explicit execution of the frozen qualifier.

Evidence flags remain open until repeated clean committed runs, exact execution
logs and artifact/linkage rejection tests are bound. This is one precondition,
not new manifest discovery, full metadata qualification, downstream cutover or
release approval. Other metadata assertions retain their own evidence.

The first local comparison assumed native policy order matched registry order
and failed. The corrected comparison matches unique policy identities instead;
no original assertion or policy changed. The diagnostic-only log is retained at
`target/rc9-completion-20260907/metadata-parent-focused-retry.log`.
