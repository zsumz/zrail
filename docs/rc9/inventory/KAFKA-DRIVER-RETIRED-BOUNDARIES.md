# Retired-tree physical boundaries

The seven `KD-RETIRED-TREE-*` assertions remain unverified. The original
`release_graph.rs:49-60` walker is still copied byte-for-byte in the physical
oracle and hash-checked against the frozen source. This slice tests that walker
and stock repository-file analysis on test-owned physical trees; it does not
run a modified filesystem oracle or claim a complete consumer cutover.

## Translation correction

The original walker considers every non-directory entry whose `Path::extension`
is `rs`, not only regular files. A real FIFO named `old.rs` exposes the difference:
the original rejects it, while the earlier `entry = "file"` proposal accepts it.
Reading a FIFO to decide this property would be incorrect and could block.

The separate [boundary proposal](../policies/kafka-driver.retired-boundary.fragment.toml)
changes only the seven tree rules to `entry = "non-directory"`. Tests compare
every other field with the original proposal and rerun all 144 ordinary parity
fixtures. The original policy and its archived evidence remain unchanged;
they must not be relabeled as proof of this correction. Broadening a zero-count
prohibition from files to non-directories is a revocation, not a grant. No root
contract or reviewed lock is changed.

## Real filesystem matrix

Each case is repeated for all seven retired roots. Fail-closed results require
`REP-FILE-006` naming the intended root, not merely any failed analysis.

| Cases per root | Original | Proposed native result |
| --- | --- | --- |
| Missing tree, empty tree, empty directory named `empty.rs` | Accept | Accept |
| Empty nested `.git` / nested `.git` containing Rust | Accept / reject | Incomplete, both |
| Contained regular-file links named `old.rs`, `old.txt`, `.rs` | Reject / accept / accept | Same |
| Contained and escaping directory links, each empty or containing Rust | Accept if empty, otherwise reject | Incomplete, all four |
| Broken links named `old.rs` / `old.txt` | Reject / accept | Incomplete, both |
| Directory-link cycle | Deliberately not executed | Incomplete |
| FIFOs named `old.rs`, `old.txt`, `.rs` | Reject / accept / accept | Same; no content reads |
| Unreadable empty directory / directory containing Rust | Accept, ignoring the read error | Incomplete, both |
| Unreadable regular Rust file | Reject without reading bytes | Reject without reading bytes |

There are 126 ordinary physical boundary cases, plus 21 permission cases that
must be run explicitly as a Unix user whose permissions are enforced. The 35
missing/empty/pruned-tree cases are portable; the remaining boundary cases need
Unix links/FIFOs. A skipped or non-Unix run does not prove those cases. Permission
restoration and external link-target cleanup are owned by RAII guards.

Integration tests additionally exercise check, coverage and lock construction:
special entries are exposed as `other`, changing a FIFO to a regular file stales
the input binding, and broken exact links cannot produce partial evidence.
Contract tests reject content predicates with the new selector; protected diff
tests cover both quantifiers, inverse changes and unproven directory relations.

## Remaining qualification

Run the ordinary suite and the separate permission cases on a qualifying Unix
host, with `TMPDIR` pointing to an external test-owned directory:

```sh
cargo test --locked --offline -p zrail-rust --lib retired_
cargo test --locked --offline -p zrail-rust --lib \
  retired_unreadable_directories_fail_closed_and_unreadable_files_still_count -- --ignored
```

Deterministic per-entry iterator-error and entry-inspection failure evidence is
still needed. Do not introduce racy deletion tests or run the legacy cycle.
Bind a fresh corrected-policy report to a clean signed producer, its executable,
the frozen source inputs and all ordinary/failure cases; repeat it and validate
artifact tampering before changing the seven ledger verification flags.

The separate `KD-TRAVERSAL-*` source-root helper has different failure semantics
and selection requirements. These retired-tree fixtures do not close its three
assertions. Frozen `support.rs:82-100` panics separately on directory reads,
entry yields and `DirEntry::file_type` failures. It never recurses directory
links, unlike the retired walker. The existing test-only copy is
`crates/zrail-rust/src/rules/size/snapshot/legacy_driver.rs`; reuse it and verify
its frozen function body instead of introducing a third interpretation.
Missing selected roots must still fail, even when other roots contain sources.
Any injected iterator-error proof must be labeled separately from the real OS
permission cases above. No downstream guards have been removed or grants accepted.
