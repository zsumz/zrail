# Rafter source-boundary and private-name review

Frozen source: `zsumz/rafter@e99b43c1b3fb3c94dd626effccee8420016594f2`.
The complete reviewed files and their SHA-256 identities are:

| Source | SHA-256 |
| --- | --- |
| `crates/rafter/tests/source_boundary.rs` | `d0d72712188265e89b5a23f8e084e733eed86ea5b918a78f1263752e8e9eacd6` |
| `scripts/private-name-scan` | `3d648d3358560b6943b4cb0d48eda6f926bab84cd53bc6b88524803e05177a0b` |
| `RELEASE.md` | `7d0a3fb991e620981eec1893983f3096baf94a703772877a9e6020f5cc6fc9c8` |

The [origin manifest](../../../crates/zrail-testkit/tests/fixtures/rc9/audited-text-origins.json)
binds byte-exact copies. Twenty-one manually reviewed shell/document spans bind
physical UTF-8 byte ranges separately from the Rust syntax census. No shell
interpreter, repository evaluator, or text-to-Rust identity inference is added
to zrail analysis.

The ledger adds **66 instances**, all unverified: 31 static boundary rules,
nine input/discovery conditions, one runner-root condition, four private-pattern
extension/detector instances, and 21 shell/document contracts. All 14 Rust census
candidates in `source_boundary.rs` have source or supporting-helper links.
Formatting and diagnostic accumulation are linked to the underlying assertion;
they are not inflated into separate architectural requirements.

## Boundaries retained

| ID family | Instances | Selected source / property |
| --- | ---: | --- |
| `RF-BOUNDARY-rafter-*` | 12 | Recursive `crates/rafter/src/**/*.rs` authored dependency-spelling prohibitions |
| `RF-BOUNDARY-rafter-app-*` | 8 | Recursive `crates/rafter-app/src/**/*.rs` authored dependency-spelling prohibitions |
| `RF-BOUNDARY-rafter-runtime-api-*` | 11 | Recursive `crates/rafter-runtime-api/src/**/*.rs` authored dependency-spelling prohibitions |
| `RF-BOUNDARY-READ-*` | 3 | Every selected file is readable as complete UTF-8 |
| `RF-BOUNDARY-TRAVERSAL-*` | 3 | Root/directory enumeration opens successfully; empty roots are permitted |
| `RF-BOUNDARY-ENTRY-*` | 3 | Every yielded directory entry can be inspected successfully |
| `RF-BOUNDARY-WORKSPACE` | 1 | The trusted runner's compile-time manifest directory has two ancestors |

For each token, the original helper rejects `token::` anywhere on a line. It
also rejects a line whose `trim_start()` begins with `use `, `pub use `, or
`extern crate ` followed by the token and then end-of-line or one of `: ; , {`,
space or tab. Matching is case-sensitive. Comments, strings, disabled cfg text,
and test files participate. This is an authored-spelling policy, not semantic
dependency identity. For example, the broader identifier `tokio_helpers` is
permitted while a comment containing `tokio::` fails. A semantic dependency ban
alone would lose this separate requirement.

The filesystem helper recursively follows `Path::is_dir()` and sorts physical
paths. It does not consult Git ignore rules or Cargo reachability. Missing roots,
enumeration errors, entry errors, and unreadable/non-UTF-8 Rust files fail.
Symlink behavior needs explicit comparison against zrail's bounded discovery;
the original traversal is not a reason to allow an unbounded analyzer walk.

Existing stock raw predicates can express one token as 22 rules: one literal
absence, three exact trimmed-line absences, and eighteen empty-allowlist prefix
rules for the boundary characters. Across the 31 tokens, this is 682 rules.
On the frozen selected families (172 / 21 / 1 files; 810,931 / 269,764 / 15,735
bytes), it consumes **253.08 MiB of the 256 MiB content-work bound** before the
remaining Rafter policies. The compact
[boundary fragment](../policies/rafter.source-boundaries.fragment.toml) uses 155
raw rules plus three exact source-root requirements. The bounded
`line-prefixes-absent` predicate reduces this slice to **57.52 MiB** of content
work, preserving the exact raw matcher. It adds no source exclusions or larger
analysis limits. Full-policy capacity and committed differential qualification
remain separate obligations.

## Private-name retirement

The 25 instances below link exclusively to the user's
[approved decision](../DECISIONS.md). Static source boundaries above, general
naming/vocabulary policy, license checks, actual package verification and
release runners retain their obligations.

| ID (`RF-PRIVATE-` prefix) | Original contract |
| --- | --- |
| `BOUNDARY-{rafter,rafter-app,rafter-runtime-api}` | Each externally supplied private pattern is absent from selected source lines. Missing/non-Unicode environment yields an empty additional pattern list. |
| `BOUNDARY-PARSER` | The exact parser test preserves the ordered three-element result after separator splitting, trimming and empty removal. |
| `SCAN-INPUTS`, `SCAN-REQUIRED-PATTERNS` | CLI/environment precedence and normalization; empty effective patterns exit 2. |
| `SCAN-ROOT`, `SCAN-RG-TOOL` | Bind the script checkout; require ripgrep rather than reporting a false clean result. |
| `SCAN-TARGETS`, `SCAN-ASSETS`, `SCAN-EXISTING-TARGETS` | Exact authored targets/globs, anchored build exclusions, optional asset pass, and existence filtering. Missing targets are not required-presence failures. |
| `SCAN-CARGO-TOOL`, `SCAN-PUBLISHABLE-SELECTION` | Require Cargo for scan coverage; choose one-level crate manifests using the original raw `publish=false` grep. |
| `SCAN-ARCHIVE-LIST`, `SCAN-ARCHIVE-NONEMPTY`, `SCAN-SCANNED-LIST` | Successful scan-specific package/rg listing pipelines and a nonempty filtered archive inventory. |
| `SCAN-ARCHIVE-CLOSURE` | Every filtered shipped path is visited by this scan's own globs. The three synthesized Cargo paths are explicit exceptions. |
| `SCAN-ARCHIVE-MODE`, `SCAN-RELEASE-NO-SKIP` | Default coverage execution; the explicit skip option warns and is normatively prohibited for release scans. |
| `SCAN-LITERAL-MATCH`, `SCAN-SEARCH-ERROR`, `SCAN-RESULT` | Case-sensitive literal search; distinguish matches, no matches and tool failure; propagate the aggregate result. |
| `RELEASE-INVOCATION`, `RELEASE-ARCHIVE-CLAIM`, `RELEASE-SCOPE-LIMITS` | Manual pre-tag invocation, scan closure claims/exceptions, and the documented limits of asset and manually maintained root selection. |

The scan does not prove that every new repository top-level path is visited.
Its raw publishability grep is not Cargo metadata validation. Retiring its
scan-specific archive coverage does not retire `scripts/reference-package-check`
or prove that packages were built or verified.

The frozen tracked-file search for `private[-_ ]name`, `private downstream`,
and `RAFTER_[A-Z_]*PATTERNS` finds only the three reviewed files. `RELEASE.md`
explicitly describes the scan as manual. The existing root CI runs
`cargo test --workspace`, which includes the Rust source-boundary integration
test; it supplies no private pattern set. The separate reference lane still
runs `scripts/reference-source-check` and requires its own policy inventory.

The proposed later Rafter cutover removes `scripts/private-name-scan`, its manual
release instructions, and only the environment/parser/private-pattern branch
of the mixed source-boundary guard under the approved retirement. The static
guard, traversal helpers, and root test invocation stay until their native
replacement is fully qualified. No downstream source is edited in this task.

## Execution evidence and limits

The trusted runner compiles the unchanged frozen `source_boundary.rs` with
`CARGO_MANIFEST_DIR` bound to `crates/rafter` and explicitly unsets
`RAFTER_SOURCE_BOUNDARY_EXTRA_PATTERNS`. Both original tests pass, with exactly
two unique outcomes and no ignored/missing/extra test. The fixture parser test
still executes its original synthetic input. This is legacy baseline evidence,
not native parity, full-workspace execution, or a private scan run.

The 66 new rows retain blockers until their replacement or retirement evidence
and final cutover linkage are bound. None increments the verified count.
