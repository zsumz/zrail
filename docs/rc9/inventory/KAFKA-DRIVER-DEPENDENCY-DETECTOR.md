# Dependency name-extraction detector

`KD-DEP-DETECTOR-EXACT-NAME` and `KD-DEP-DETECTOR-PRESENT-NAME` are the two
assertions in frozen `dependency.rs:287-295`. They exercise an inline string,
not the live Cargo graph. The complete original function is copied unchanged;
the already imported `lockfile_packages` implementation is reused unchanged.
Both excerpts bind the frozen file and exact function bytes.

The [fixture policies](../policies/kafka-driver.dependency-detector.fixture.toml)
use stock trimmed, case-sensitive `line-absent` and `line-present` predicates
over a temporary `detector.txt`. They are qualification fixtures, not additional
consumer requirements: do not add a new live requirement that Cargo.lock must
contain `bytes`, or install this fixture bundle during downstream cutover.
The original raw bans have separate existing qualification.

The 32-case matrix checks full extracted name sets and both native predicates:
the original fixture, exact additions/removals, similar names, Unicode trimming,
CRLF, no final newline, duplicate lines beyond the offset sample, comments,
assignment spelling, case, unrelated sections, malformed TOML, multiline text,
embedded quotes/backslashes, non-Rust whitespace, bare CR and NUL decoration.
Raw matching intentionally does not become TOML or package-graph interpretation.
Original sets deduplicate names; native line counts preserve multiplicity and
only sample the first sixteen matching offsets. Membership decisions must agree.

`scripts/rc9-dependency-detector` executes the original detector, source binding,
ordinary matrix and explicit frozen qualifier from a clean committed producer.
Snapshot and excerpt checks run before and after execution.

## Bound qualification, 2026-09-09

Signed producer `2a066fe2c9588bab3ccf737e0ff830111d7f9b79`, tree
`9e14b141c64f48c7e60b4dae58dd87ebbd61246d`, produced two byte-identical reports
with payload SHA-256
`1b690baa1b241d2133f1dfbbecb23bc7ed8d7084171c111418431615a0645feb`.
Each run executes four exact tests, including the original inline function and
explicit frozen qualifier. Each report records 32 cases and 64 native policy
comparisons; full original name sets, input bytes, counts and sampled offsets
are bound. The ordinary matrix also runs independently in each invocation.

The [index](../evidence/dependency-detector-index.json) binds both reports and
all 22 report/log files, original source/excerpts, policy/matrix, compiler,
executable, clean producer and unchanged snapshots. Six validator tests reject
92 mutations and two runner tests reject 28 malformed outcomes. Both ledger
entries now bind the corresponding fixture policy and `REP-FILE-004`, with
`full_snapshot_verified = false`. No downstream guard, root authority or release
status changes here.
