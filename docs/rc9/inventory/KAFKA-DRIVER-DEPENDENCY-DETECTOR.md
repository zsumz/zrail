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
Snapshot and excerpt checks run before and after execution. Verification flags
remain open until repeated reports, exact execution logs and tamper validation
are bound. No downstream guard, root authority or release status changes here.
