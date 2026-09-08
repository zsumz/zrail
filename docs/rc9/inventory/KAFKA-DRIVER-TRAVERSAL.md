# Source-traversal preconditions

The three `KD-TRAVERSAL-*` rows are separate from the retired-tree rules.
Frozen `support.rs:82-100` fails on directory reads, entry yields and
`DirEntry::file_type` errors. The existing `legacy_driver.rs` copy is reused
unchanged; its complete function digest is checked against the frozen extraction.
The six roots come from the frozen `guardrails.toml`; they are nonoverlapping.
The original sorts entries and final paths but does not deduplicate roots.

## Proposed closed policy

The [partial fragment](../policies/kafka-driver.traversal.fragment.toml) uses two
existing file predicates. `kd-source-roots` requires exactly the six directory
paths, so losing one empty root fails `REP-FILE-002` even when another root has
Rust files. `kd-source-traversal` selects every physical entry beneath those
roots with `entry = "any"` and a zero minimum. Empty trees are allowed, but the
selection still requires complete inspection; it is not a vacuous source count.
Read/entry/inspection and unread-descendant failures remain `REP-FILE-006`.
No root contract, lock or rc8 source-root semantics change.

The original helper inspects non-Rust entries and empty directories too. It
does not follow directory links encountered during recursion, but it follows a
configured root that is itself a directory link. Non-directory `.rs` paths,
including symlinks and FIFOs, enter its physical source list without reading
their contents. The native physical observation list preserves those paths
where analysis is complete. This does not prove that later Rust parsing accepts
special entries, or qualify the separate registry parser/configuration migration.

## Explicit differences and matrix

Native glob completeness refuses pruned `.git` descendants, broken/escaping
links and directory-link descendants. Its repository-wide scan also refuses an
unread directory outside the six roots. Those are stronger failures than the
old helper, never exact error-path parity. The old helper's directory-link cycle
terminates because it does not recurse links; this old cycle is safe to execute,
unlike the retired-tree walker. Native traversal remains bounded and UTF-8-path
constrained; this is not unbounded or arbitrary-config equivalence.

- 42 portable cases: each root missing, replaced by a file, empty, with nested
  empty directories, ordered/cfg/extension/decoy inputs, and empty/Rust `.git`.
- 66 Unix cases: eight link cases and three FIFO extension cases per root.
- 19 explicitly permission-gated cases: unread empty/nested directories and
  unread source files per root, plus an unread directory outside selected roots.
- 96 injected cases: six roots, empty/nonempty nested directories, two error
  kinds, and directory/first-entry/last-entry/metadata failures.

The injected original failure expressions retain their exact frozen clauses.
They consume explicitly injected `Result` values or a test-only entry wrapper;
these are not observed OS `DirEntry` errors. Native proofs use the existing
static filesystem seam and require an exact error from the entire scan, with
one injected operation and no partial inventory. Its metadata inspection is
not misrepresented as a call to `DirEntry::file_type`. Real permission/link
cases separately bind the public file-analysis diagnostic path.

The proposed trusted runner is `scripts/rc9-traversal`. Qualification must use a
clean signed producer, bind the complete frozen selected-path/input inventory,
execute permission cases explicitly, and archive two byte-identical reports
with exact execution logs and tamper validation before changing verification
flags. No complete consumer policy, downstream guard deletion, authority
acceptance or release approval follows from these three preconditions.
