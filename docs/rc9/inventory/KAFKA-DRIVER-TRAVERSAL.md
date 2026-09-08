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

- 48 portable cases: each root missing, replaced by a file, empty, with nested
  empty directories, ordered/cfg/extension/decoy inputs, component-order
  differences, and empty/Rust `.git`.
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

The first frozen run at `08696c81e44542a92be2ea7ff4fe15b01edac8a6` found all
772 paths in both inventories, but its direct vector comparison failed: native
UTF-8 string ordering puts `a.rs` before `a/nested.rs`, while the original
`PathBuf` ordering reverses them. The failed log is preserved, not relabeled as
a pass. Both orders are retained in the corrected report and compared as
multisets without deduplication. Six explicit cases enforce this distinction;
the precondition proof does not assert identical output ordering.

## Bound qualification

The trusted `scripts/rc9-traversal` runner produced two byte-identical reports
from clean signed producer `ec8ac46392ab08e2b6cf80c3fb944961e89b52d7`, tree
`39831085492981052d680632c9db2fc60e2d86d3`. Each explicitly executes seven tests,
including the permission gate and frozen qualifier. All 772 frozen Rust inputs
and 846 physical entries are bound, alongside the 133 physical and 96 injected
cases above. No snapshot or producer input changed during either run.

The [index](../evidence/traversal-index.json) binds producer, compiler, executable,
policy, both full reports and all 28 successful execution-log/report files.
Payload SHA-256 is
`8f38573e3888798cbff42bb023cc8315c9a60096c023df35ce56d344d61777e1`.
The initial failure logs are archived separately and confer no verification.
Eight validator tests reject 93 artifact, structural, outcome and ledger-linkage
mutations; two runner tests reject 42 malformed execution outcomes. Structural
checks independently reconstruct frozen paths from the tracked-file census and
require every physical and injected case, not just a self-reported case count.

The three ledger rows now bind both policies as one precondition bundle and
`REP-FILE-002`/`REP-FILE-006`, with `full_snapshot_verified = false`.
The subsequent binding revision makes the report type platform-independent
while keeping Unix execution gated. Self-analysis had rejected the first field
of the conditionally declared report with `RUST-INCLUDE-002`; the revised type
has zero unresolved bindings. No case, serialized field, policy or analyzer
rule changes. Archived reports retain their exact historical producer identity;
they do not claim that producer passed the complete workspace gate.
The conditional-report binding limitation remains an analyzer follow-up; this
test layout change is not a general engine fix for conditional type bindings.
No complete consumer policy, downstream guard deletion, authority acceptance
or release approval follows from these three preconditions.
