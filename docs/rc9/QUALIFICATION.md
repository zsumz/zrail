# Reproducing the current rc9 evidence

Release verdict: **blocked**. The completed trait-set evidence uses
`53936654ffbaff1bb0e9e127db46d3ca0aa510ba`; its archived canonical gate fails
only the five pending protected-lock diagnostics. Import-rename evidence uses
`405fe5674f77b8a31bd5aea01b190da55c7dcbcf`.
Qualification-text parity retains its `7c6c01d0af8bdeee598e87c3c8fae97ce72b9a0c`
identity, based on the exact audited rc8 commit. Whole-lock parity retains its `5251b4f056741768eb667c74d6e0949257ca843b`
identity. Manifest-field parity retains its
`c7ff4588f029169c10cce191be3bac3ceaec9e2d` identity. Earlier evidence retains its
own implementation identity below. None constitutes final versioned-tree release
qualification. The workspace version and internal pins remain `0.0.3-rc.8`.

## Current pending Rafter boundary slice

[The bound evidence](evidence/rafter-boundary-index.json) identifies HEAD
`01bbf38481217b0611a387ee2af488328f52137c` plus its exact tracked diff and every
untracked input hash. No input changed during the run. The code is pending
commits because the configured GPG signing key is unavailable; this is not a
clean committed release qualification.

The final `scripts/check` run passes artifact validation, formatting, strict
Clippy, **1,609 Rust tests** (zero failures; 17 explicitly ignored), and rustdoc.
Self-analysis is complete: **1,051 files, 1,646 base contexts, zero derived,
1,194,894 projection work, zero unresolved**. It fails at the five protected
lock diagnostics `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`, `LOCK-030`.
Its archive/cleanliness stages are consequently unreached. Separate
`scripts/package-check`, `git diff --check` and staged whitespace checks pass.
The original failed gate and its corrected diagnostic assertion are preserved
in the index; no failed run is relabeled as passing.

The native Rafter boundary translation passes **992** focused original/native
comparisons: **248 accepted / 744 rejected**, each checked against its intended
policy/category. All **194** frozen family inputs pass the original matcher and
the 158-rule native file slice, within the unchanged work bound. This does not
establish full source/Cargo analysis, execution receipts, repeated per-case
evidence at a committed revision, or Rafter cutover readiness.

With the environment below, reproduce the relevant checks using fresh outputs:

```sh
python3 scripts/rc9-source-boundary-policies /absolute/snapshots /fresh/rafter.fragment.toml
cargo test --locked --offline -p zrail-core --lib prefix
cargo test --locked --offline -p zrail-rust --lib boundary_sources
cargo test --locked --offline -p zrail-testkit --test repository_files
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots cargo test --locked --offline -p zrail-rust --lib frozen_rafter_boundary_files_fit_the_work_bound_and_match_the_original -- --ignored
scripts/check
scripts/package-check
```

Compare the generated fragment byte-for-byte with
`docs/rc9/policies/rafter.source-boundaries.fragment.toml`. The canonical gate
remains expected to fail until protected review is complete; running the archive
check separately does not bypass that release decision.

The [public API/docs baseline](evidence/rafter-public-api-legacy-index.json)
compiles the unchanged frozen guard with the pinned compiler and correct
manifest context. All eight actual top-level tests pass; missing, extra,
duplicate or ignored outcomes are rejected. A synthetic `unit` test inside a
raw-string detector fixture is not an executed test. The source-boundary
baseline similarly passes its two original tests with private-name injection
explicitly unset. These are source-policy detector baselines, not simulation,
Maelstrom, consumer builds, or native replacement qualification.

## Committed transport evidence and census checkpoint

Commit `11b4e70d5db2f6d3d5d6d55237debe8606058814`, tree
`37810e99b6fb01f1af4cad165b8e96655fc1f95e`, was qualified in a clean,
isolated checkout with the unchanged reviewed root contract and lock.
[The evidence index](evidence/transport-evidence-index.json) binds the logs.
`scripts/check` passed artifact validation, eight Python unittest methods,
formatting, Clippy, 1,560 Rust tests (13 explicitly ignored), and rustdoc.
Self-analysis was complete: 1,020 files, 1,611 base contexts, 1,174,167 work,
zero unresolved. It rejected the five pending authority differences:
`LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`, and `LOCK-030`.
The script therefore did not reach its archive or cleanliness steps.
Standalone `scripts/package-check` passed, and an explicit final Git status was
clean. No authority was accepted.

The initial `d4f901c` run additionally rejected three unreviewed macros in the
new census test fixtures. Those helpers now use ordinary JSON parsing and the
existing reviewed panic syntax; the failed log is retained. This checkpoint
does not relabel earlier frozen differential evidence as having run on this
revision. Full final-release and downstream qualification remain outstanding.

## Environment and trusted setup

The runs use the repository-pinned Rust **1.97.1**, on Linux, with locked Cargo
dependencies. Prefetch is a trusted setup operation. The stock analyzer never
executes Cargo, repository programs, or network operations.

```sh
# Choose isolated paths with sufficient disk space.
export CARGO_TARGET_DIR=/absolute/rc9-target
export CARGO_BUILD_JOBS=2
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_PROFILE_DEV_DEBUG=0
export TMPDIR=/absolute/rc9-evidence

cargo fetch --locked
python3 scripts/rc9-snapshots /absolute/snapshots --prefetch
python3 scripts/rc9-snapshots /absolute/snapshots
```

All consumer checkouts must be clean and match `snapshots.json`. The setup tool
does not reset an existing checkout. Qualification mutations belong in separate
copies. No downstream snapshot was patched in these runs.

## Historical facade qualification

The following table describes implementation
`348ae5b777217f774433045da5c4431ea055c162`. Its logs and index remain immutable
historical evidence; the newer size-policy qualification is recorded below.

| Check | Result and precise limit |
| --- | --- |
| Untouched rc8 `scripts/check` in a separate detached worktree | **Pass**, exit 0. Formatting, lint, tests, rustdoc, self-hosting, normalized archives, and cleanliness checks completed. 1,389 passing test outcomes; one existing advisory timing test ignored. |
| rc9 `scripts/check` at the tested implementation | **Fail**, exit 1 at self-hosting. Structure, formatting, strict workspace lint, workspace tests, and rustdoc passed. 1,406 passing test outcomes; zero test failures. |
| Final source analysis | **Complete**: 862 Rust files, 1,391 base contexts, zero derived contexts, 1,157,810 projection work, zero unresolved analysis items. This is zrail self-analysis, not downstream qualification. |
| Final self-hosting diagnostics | Exactly `LOCK-008`, `LOCK-026`, `LOCK-028`, `LOCK-030`: analyzer epoch, changed reviewed `CHANGELOG.md` gate input, analyzed inventory, and certificate epoch. No remaining source-policy violation. |
| Standalone `scripts/package-check` | **Pass**, exit 0. Existing normalization, license/README equality, exact local dependency archive staging, and offline expanded-archive checks completed. This does not turn the interrupted canonical gate into a pass. |
| Strict facade acceptance | **8 tests pass**, including imported frozen Kafkars counterexample, required diagnostic categories, strict item rejection, test identity/reachability, missing mounts, production leakage, and expression fragments. |
| Frozen facade differential predicate test | **Pass**, 28 inputs × 2 consumer predicates = 56 comparisons. Origins and extracted function bytes are independently checked against the frozen snapshots. This does not establish consumer-wide selection parity. |
| Frozen kafka-driver legacy guard target | **47/47 pass**, zero ignored/filtered, exact unique outcome inventory. Unmodified `tests/guardrails.rs` was compiled as a trusted minimal harness; runtime, broker, simulation, and archive programs were not thereby qualified. |
| Repeated all-file census | **Pass**, two fresh byte-identical census runs and three focused discovery tests. 9,792 tracked files and 79,949 syntax candidates. Discovery is not completed assertion review. |
| Audit completeness gate | **Expected failure**, exit 1: assertion inventory remains incomplete. |
| Strict rc8 same-revision epoch migration | **36 preserved** entries; zero retired, newly observable, or changed interpretation entries. No lock acceptance was performed. |
| Protected diff against rc8 | Zero grants/debt changes, **2 unknowns**: the unchanged before/after locks both use epoch 6. These unknowns remain protected; the ordinary diff is not migration approval. |
| Full consumer checks, coverage, receipts, and cutover qualification | **Not completed** for any repository or independent workspace. No complete policy bundles exist yet. |

The normal workspace test run leaves two tests ignored: the existing advisory
wall-clock smoke, and the explicitly prefetched census. The latter was run
separately with `--include-ignored`; it passed twice with identical output.

The frozen kafka-driver harness uses its exact locked `syn` 2.0.119, `serde`
1.0.229, and `toml` 0.8.23 versions from the zrail build. Its receipt records the
actual feature sets, toolchain, compile command, rlib/binary hashes, source pin,
and all test outcomes. Zrail's syn feature set additionally includes
`extra-traits`. The harness does not claim a native downstream Cargo workspace
build or qualification on kafka-driver's older CI toolchain.

## Scoped-size qualification

The committed size report covers 772 kafka-driver and 6,280 Kafkars source files,
21,300 positive/negative comparisons, 141 exact baseline instances, and three
hard allowances. Full frozen traversal and native physical inventory select the
same sets without duplicate files. Every report row includes its source digest,
effective policy, measurements, and intended diagnostic IDs. The report is
strictly size-only; no partial lock or source-resolution certificate is produced.

Its uncompressed SHA-256 is
`f060ab308686ba55c1a955d7aaa8660cd3f960e495b6ce87b6ba4fb8cc39f37b`.
The implementation was clean when this report ran. Kafkars' and Rafter's
unmodified frozen size targets additionally passed five tests each; receipts
record their original source context, pinned compiler dependencies and outcomes.
The private-name retirement did not modify any frozen input.

At `cb198bada58ef8f08bdc60bc7ac5cea0b2993a0a`, the canonical gate passed
structure, formatting, strict workspace lint, **1,439 tests**, and rustdoc, then
stopped at exactly `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030`. Self-analysis
was complete: 886 physical Rust files, 1,426 base contexts, 1,177,203 projection
work, and zero unresolved items. The three normal-run ignores are the existing
advisory timing test and the two explicit-prefetch qualification tests.

The final size report was repeated twice in a clean detached worktree at that
same commit, producing identical bytes. Standalone `scripts/package-check`
also passed there, and its checkout remained clean. Archive and cleanliness
stages after self-hosting were not reached by the canonical gate; the separate
archive result does not make that gate pass. See [the current evidence index](evidence/size-index.json).
Earlier size evidence remains under `evidence/size-767abfd/`.

After the same trusted prefetch and environment setup:

```sh
python3 scripts/rc9-size-policies /absolute/snapshots /absolute/rc9-size-fragments
diff -u docs/rc9/policies/kafka-driver.sizes.fragment.toml /absolute/rc9-size-fragments/kafka-driver.sizes.fragment.toml
diff -u docs/rc9/policies/kafkars.sizes.fragment.toml /absolute/rc9-size-fragments/kafkars.sizes.fragment.toml
cargo test --locked --offline -p zrail-testkit --test scoped_budgets --test strict_facades --test test_facade_discovery
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_SIZE_REPORT=/absolute/rc9-evidence/size-parity.json
cargo test --locked --offline -p zrail-rust --lib qualify_all_frozen_kafka_size_instances -- --ignored --nocapture
python3 scripts/rc9-legacy-kafka-driver /absolute/snapshots/kafkars /absolute/rc9-evidence/kafkars-sizes --surface kafkars-sizes
python3 scripts/rc9-legacy-kafka-driver /absolute/snapshots/rafter /absolute/rc9-evidence/rafter-sizes --surface rafter-sizes
```

The report output must be new. Repeating the same committed tree and inputs
produces identical bytes. Across commits the recorded implementation identity
changes even when the measurement rows remain identical.

## Repository-file engine qualification

At `b32ee44d4dabf854977e42de98d0bcb5d8d8ca30`, `scripts/check` passed
structure, formatting, strict workspace lint, **1,469 tests** (zero failures,
three explicit ignored tests), and rustdoc. Complete self-analysis recorded
910 Rust files, 1,469 base contexts, 1,202,401 projection work, and zero unresolved
items. The gate stopped at the same four protected lock diagnostics:
`LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030`. No source violation remained.
The standalone `scripts/package-check` subsequently passed on that clean tree.
The archives retain the rc8 version and are not release-qualified assets.

The 14 file-policy integration tests exercise every predicate, intended policy
identities/diagnostics, changed-but-still-valid input binding, exact-set exchanges,
missing positive subjects, persistent prohibitions, raw matching positions,
non-overlapping quantities, excluded sources, bounded reads, and symlink/pruning
completeness. Nine protected-diff tests and three new glob-prefix proof tests
also passed in the canonical run. These are engine tests; frozen consumer file
assertions are not yet marked verified on their basis. See
[`files-index.json`](evidence/files-index.json) for code identity and log hashes.

```sh
cargo test --locked --offline -p zrail-testkit --test repository_files
cargo test --locked --offline -p zrail-core diff::files
cargo test --locked --offline -p zrail-core path::
```

## Frozen Kafka-driver file-predicate parity

Implementation `ba7a52337ec435285e23a8ee1d64a095e90ab8e2` compares 39 complete
physical/raw policy selections and 3,468 frozen path outcomes, then evaluates
194 independent physical fixtures with the stock file analyzer and frozen raw
helpers. Every policy has positive and negative cases, with the intended policy
ID and diagnostic required. Directory counterexamples execute the legacy
`is_dir` predicate. Source and registry pins, extraction bytes, implementation
commit, compiler version, test-binary hash, Cargo.lock hash, and fixture contexts
are bound in the report. Both source and implementation must remain clean.

Run from the committed implementation, with the external build environment above:

```sh
python3 scripts/rc9-file-policies /absolute/snapshots /absolute/policy-preview
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_FILE_REPORT=/absolute/rc9-evidence/file-parity-a.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_file_predicates -- --include-ignored --nocapture
export ZRAIL_RC9_FILE_REPORT=/absolute/rc9-evidence/file-parity-b.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_file_predicates -- --include-ignored --nocapture
cmp /absolute/rc9-evidence/file-parity-a.json /absolute/rc9-evidence/file-parity-b.json
```

Use fresh report paths in the same external directory; the fixture context stays
identical and is removed after each case. Outputs inside implementation or
snapshot repositories are rejected. The recorded repeated payload is
`edf2b1d64d99176a7cf53695fe0abfd8c9e3d186a146ffdf2a490e9a4cfaec63`.
[`file-parity-index.json`](evidence/file-parity-index.json) links the compressed
report and logs. The inventory integrity gate verifies each of the 44 linked
assertion instances against report identity and positive/negative outcomes.
The new explicitly prefetched test is ignored by default, so ordinary workspace
qualification now leaves four tests ignored; release qualification must execute
all applicable prefetched suites separately.

## Other reproduction commands

Run the baseline in an isolated checkout of
`5a368379360104ca19745326cfcef48d22a6452b`, with its own target directory:

```sh
scripts/check
```

Run the following from the rc9 implementation checkout after trusted prefetch:

```sh
scripts/check
scripts/package-check
cargo test --locked --offline -p zrail-testkit --test strict_facades
cargo test --locked --offline -p zrail-rust frozen_facade_predicates_agree_with_stock_wiring_modes
python3 scripts/rc9-legacy-kafka-driver /absolute/snapshots/kafka-driver /absolute/rc9-evidence/kafka-driver

export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_CENSUS=/absolute/rc9-evidence/census.json
cargo test --locked --offline -p zrail-rust --test rc9_inventory -- --include-ignored --nocapture
python3 scripts/rc9-inventory-check --census /absolute/rc9-evidence/census.json
python3 scripts/rc9-inventory-check --require-complete

cargo run --quiet --locked --offline -p zrail --bin zrail -- coverage --format json
cargo run --quiet --locked --offline -p zrail --bin zrail -- diff --base 5a368379360104ca19745326cfcef48d22a6452b --format json
```

The full gate and audit completeness command currently return nonzero for the
reasons above. Do not remove those checks, discard the reviewed lock, or accept
authority merely to obtain a successful exit.

For a fresh migration preview, create an untracked output directory and choose a
new report path; the CLI correctly refuses to overwrite a tracked report:

```sh
mkdir -p .zrail/cache/rc9-review
cargo run --quiet --locked --offline -p zrail --bin zrail -- migrate-lock \
  --base 5a368379360104ca19745326cfcef48d22a6452b \
  --output .zrail/cache/rc9-review/epoch-7.json
```

The archived preview is `evidence/rc8-migration.json`, digest
`6594cbd483844fa7246c69861c2fd7db5d105e9e11cbdde0c00644e966b81a0f`.
Cross-revision recovery is unnecessary here: rc8 remains fully analyzable, so
the existing CLI correctly requires strict same-revision migration.

## Authority and evidence boundaries

[`AGENTS.md`](../../AGENTS.md) explicitly says:
“Never run `zrail update --accept-grants`, weaken `zrail.toml`, or replace
`zrail.lock` without explicit human authorization.” The reviewed root contract,
lock, versions, and exact internal pins remain unchanged. A migration preview is
review material, not authority acceptance.

The evidence index records log and artifact identities. No downstream guard
deletion, policy weakening, release tag, push, publication, or release asset
upload was performed. The source remediations made while developing this slice
were confined to zrail: its facade line budget and trusted fixture/census input
handling were fixed without changing self-governance policy.

## File parity canonical rerun

At clean commit `338eb74958bb872d4ee8a017e44adb7dd97d2ebf`, `scripts/check`
passed structure, formatting, strict workspace lint, **1,470 tests** (zero
failures, four explicit ignores), and rustdoc. Complete self-analysis covered
915 Rust files, 1,474 base contexts, and 1,203,673 projection work, with zero
unresolved items. It stopped at `LOCK-008`, `LOCK-026`, `LOCK-028`, and
`LOCK-030`; archive and cleanliness stages were not reached. The immutable
[run index](evidence/files-parity-canonical.json) binds the full compressed log.
This run predates the UTF-8 equality extension. That extension passed its
15 integration tests, ten protected-diff tests, and strict workspace lint;
it still needs the subsequent complete gate.

## Authored document engine qualification

At `8e37c7203be2c1d97b149b93a4711b7214c09e92`, the complete gate passed
structure, formatting, strict workspace lint, **1,484 tests** (zero failures,
four explicit ignores), and rustdoc. Complete self-analysis covered 924 Rust
files, 1,490 base contexts, 1,211,785 projection work, and zero unresolved items.
The same four lock-review diagnostics stopped self-hosting. Archive and
cleanliness stages were not reached; no lock was accepted.
The [document evidence index](evidence/documents-index.json) binds the exact log.
The initial gate at `04d1a72` had stopped at a misplaced test file; that failure
was corrected before the reported run.

The nine document integration tests cover typed field selection, exact arrays,
Unicode trimming, absent/null distinctions, wrong intermediate types, full-input
lock binding, repeated coverage, duplicate-key parsing, invalid UTF-8, and
resource exhaustion. Two protected-diff tests cover changed exact values and
order, weakened predicates, removed guards, and unproven selector changes.
A strict-schema test rejects unsupported formats, values, expressions, and
oversized expectations. These engine results do not close downstream assertions
until their individual frozen differential evidence is linked.

## Frozen metadata parity and canonical checkpoint

At clean commit `db11ebe40f3ffe969939785b34f2acf880c6b79e`, two frozen metadata
runs produced byte-identical reports with SHA-256
`9895d7213b58d96e144fdc362c6c9fc0d86274f8b93ebdb47b1bfe9423340485`.
They cover 35 policies, 11 frozen inputs, and 134 physical cases: 35 accepted
and 99 rejected through the intended policy diagnostics. Thirty-three assertion
IDs now bind this evidence. The parent-path helper's proof linkage remains open.

The original metadata assertion body and helpers execute in trusted
qualification; only the workspace root is supplied from an isolated input path.
Every negative case changes one selected input, preserves all other input
bytes, and checks the original guard plus the intended native rule. Malformed,
duplicate-key, and invalid-UTF-8 cases require explicit incomplete-analysis
failure. The report includes all fixture input hashes, entry kinds, original
failure messages, source/policy/code/compiler identities, and complete native
observations. See the [metadata index](evidence/metadata-index.json).

The same clean commit passed structure, formatting, strict workspace lint,
**1,485 tests** (zero failures, five explicit ignores), and rustdoc. Self-analysis
was complete: 930 Rust files, 1,496 base contexts, 1,213,212 projection work, and
zero unresolved items. The canonical gate stopped at `LOCK-008`, `LOCK-026`,
`LOCK-028`, and `LOCK-030`; archive and cleanliness stages were not reached.
Standalone `scripts/package-check` subsequently passed on that same clean tree.
Its rc8-version archives are experimental qualification outputs, not rc9 assets.

After the trusted prefetch and environment setup described above:

```sh
python3 scripts/rc9-metadata-policies /absolute/snapshots /absolute/new-metadata.fragment.toml
diff -u docs/rc9/policies/kafka-driver.metadata.fragment.toml /absolute/new-metadata.fragment.toml
cargo test --locked --offline -p zrail-testkit --test repository_documents --test repository_files
cargo test --locked --offline -p zrail-core --lib files
cargo test --locked --offline -p zrail-rust --lib frozen_metadata_assertions_agree
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_METADATA_REPORT=/absolute/evidence/new-metadata-parity.json
cargo test --locked --offline -p zrail-rust --lib qualify_all_frozen_kafka_driver_metadata_assertions -- --ignored
python3 scripts/rc9-inventory-check
```

The generated fragment and report outputs must be new. Repeat the metadata qualification command
with a second fresh report path on the same clean commit to compare canonical
JSON bytes. The regular unit suite runs the portable 134-case matrix without
prefetch; the explicitly ignored runner additionally qualifies the full frozen
metadata selection and immutable source identities. Neither claims complete
Cargo/Rust analysis, downstream execution, or full-repository replacement.

## Exact line engine qualification

At clean commit `f058429466c1bc6349c422e1d207140502b84ad4`, `scripts/check`
passed structure, formatting, strict workspace lint, **1,490 tests** (zero
failures, five explicit ignores), and rustdoc. Complete self-analysis covered
931 Rust files, 1,497 base contexts, 1,215,507 projection work, and zero unresolved
items. Only `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030` stopped the gate.
Archive and cleanliness stages were not reached. The
[line evidence index](evidence/lines-index.json) binds the full compressed log.

Three new integration tests cover exact line boundaries, Unicode trimming,
LF/CRLF handling, duplicate-line quantities, display omission, persistent bans,
and missing positive inputs. Strict validation rejects impossible normalized
line literals; protected comparisons distinguish exact-line identity changes
from weakened whole-file substring checks. These engine tests do not yet
close the corresponding frozen dependency/attributes assertions.

## Authored key-set engine qualification

At clean commit `b846d4e5edfdb9ae3c293c3295eccfb81724d4f1`, `scripts/check`
passed structure, formatting, strict workspace lint, **1,495 tests** (zero
failures, five explicit ignores), and rustdoc. Complete self-analysis covered
933 Rust files, 1,500 base contexts, 1,218,510 projection work, and zero unresolved
items. Only `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030` stopped the gate.
Archive and cleanliness stages were not reached. The
[key-set evidence index](evidence/key-sets-index.json) binds the complete log.
The initial gate at `4c4407c` stopped at a Clippy pattern-style error; the
qualified revision fixes it without changing predicate semantics.

The new integration fixtures cover exact and allowed immediate key sets,
equal-count substitutions, missing subjects, explicit present-non-table empty
projections, bounded display samples, and deterministic observations. Protected
diffs compare actual accepted sets and type permissions. These engine tests
do not by themselves close frozen dependency assertion IDs.

## Frozen authored dependency key parity

At clean commit `c86f5173c90e564b204420789faf2d16ef17807a`, two independent
trusted runs produced byte-identical JSON with SHA-256
`09651c2e17559c8f0e445a5f79f29781858898bbae4f9bf0c2b7130ee1466d6b`.
Five native policies accept all five frozen manifests, with the original registry
also bound. The 130 fixtures comprise 22 accepted and 108 rejected outcomes;
every original dependency key is independently removed and aliased. Native
violations use the intended policy and `REP-FILE-007`; malformed/duplicate/UTF-8
inputs fail through `REP-FILE-006`. Every old/new acceptance outcome agrees.

The same clean commit passed structure, formatting, strict workspace lint,
**1,496 tests** (zero failures, six explicit ignores), and rustdoc. Complete
self-analysis covered 937 Rust files, 1,504 base contexts, 1,219,301 projection
work, and zero unresolved items. Only `LOCK-008`, `LOCK-026`, `LOCK-028`, and
`LOCK-030` stopped the gate; archive and cleanliness stages were not reached.
See the [key parity index](evidence/key-sets-parity-index.json).

After the prefetched snapshot and external build setup above:

```sh
python3 scripts/rc9-key-set-policies /absolute/snapshots /absolute/new-keys.fragment.toml
diff -u docs/rc9/policies/kafka-driver.key-sets.fragment.toml /absolute/new-keys.fragment.toml
cargo test --locked --offline -p zrail-rust --lib frozen_dependency_key_sets_agree
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_KEY_SETS_REPORT=/absolute/evidence/new-key-sets-parity.json
cargo test --locked --offline -p zrail-rust --lib qualify_all_frozen_kafka_driver_dependency_key_sets -- --ignored
python3 scripts/rc9-inventory-check
```

Repeat with a second fresh report path on the same clean revision. The trusted
runner reuses metadata parity input binding and isolation; its callbacks exist
only in zrail's own compiled tests. Runtime analysis executes no legacy checker.
The minimal registry projection is fixed by six bound inputs and does not claim
replacement of the separate registry-parser assertions. No consumer source or
reviewed authority changed, and this is not full repository qualification.

## Frozen manifest-field parity and archives

At clean commit `c7ff4588f029169c10cce191be3bac3ceaec9e2d`, two trusted runs
produced byte-identical JSON with SHA-256
`bdf99f0a4fe71e27f5830fe565e5bb84cf26cf8ea2be59a18986fd36c2cfb1f6`.
Thirty native policies accept all frozen field selections, with five manifests
and the original registry bound. The 151 fixtures comprise 34 accepted and
117 rejected outcomes, all agreeing with the original assertion bodies/helpers.
The [manifest-field index](evidence/dependency-fields-index.json) links 35
reviewed IDs, including explicit conjunction proofs for publication and feature
array preconditions. The native typed-field projection has separate strict
schema, adversarial, coverage, and protected-diff tests.

The same clean commit passed structure, formatting, strict workspace lint,
**1,501 tests** (zero failures, seven explicit ignores), and rustdoc. Complete
self-analysis covered 944 Rust files, 1,511 base contexts, 1,221,288 projection
work, and zero unresolved items. Only `LOCK-008`, `LOCK-026`, `LOCK-028`, and
`LOCK-030` stopped `scripts/check`; archive and cleanliness stages were not
reached within that command. Standalone `scripts/package-check` then passed
on the same clean tree, verifying the three extracted package archives. Their
version remains rc8; these are experimental qualification outputs, not rc9 assets.

After the prefetched snapshot and external build setup above:

```sh
python3 scripts/rc9-dependency-field-policies /absolute/snapshots /absolute/new-fields.fragment.toml
diff -u docs/rc9/policies/kafka-driver.dependency-fields.fragment.toml /absolute/new-fields.fragment.toml
cargo test --locked --offline -p zrail-testkit --test repository_documents
cargo test --locked --offline -p zrail-core --lib files
cargo test --locked --offline -p zrail-rust --lib frozen_dependency_fields_agree
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_DEPENDENCY_FIELDS_REPORT=/absolute/evidence/new-dependency-fields-parity.json
cargo test --locked --offline -p zrail-rust --lib qualify_all_frozen_kafka_driver_dependency_fields -- --ignored
python3 scripts/rc9-inventory-check
scripts/check
scripts/package-check
```

Repeat with a second fresh report path on the same clean revision. The trusted
qualification code shares existing file-analysis isolation and input binding;
none of its legacy evaluator bodies execute during stock zrail analysis. Full
downstream Cargo/Rust and execution qualification remains outstanding.

## Native whole-lock inventory qualification

At `bd6372b91d39f59c3064a7a327b1b4349be7e28a`, `scripts/check` passed
structure, formatting, strict workspace lint, **1,514 tests** (zero failures,
seven ignored), and rustdoc. Complete self-analysis found **956 Rust files**,
1,531 base contexts, no derived contexts, 1,230,844 projection work, and zero
unresolved items. Only `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030` remain.
The canonical gate stopped before its archive and cleanliness stages.

Standalone `scripts/package-check` passed on the same clean revision. The
[index and bound logs](evidence/lock-packages-index.json) preserve both results.
No reviewed lock was changed, and the archives still carry the development
rc8 version; these are qualification artifacts, not release assets.

The new family has six core schema/protected-diff tests and seven integration
tests covering unreachable lock nodes, zero bans, required-node deletion,
same-count version/source/checksum substitutions, exact local identities,
missing/duplicate/malformed graphs, repeated selector work bounds, complete
counts despite omitted identity samples, deterministic coverage/explanation,
and whole-input lock drift. Reproduce with:

```sh
cargo test --locked --offline -p zrail-core lock_packages
cargo test --locked --offline -p zrail-testkit --test lock_packages
scripts/check
scripts/package-check
```

Consumer provenance policy translation and frozen differential qualification
remain separate work. This native checkpoint adds no verified assertion IDs
and establishes no full consumer repository qualification.

## Authored provenance differential qualification

The two clean runs at `51901c18681e15b1f14db2b61a2db057c678a90d accepted
all 27 policies on frozen Kafka-driver and produced identical
`f7b83df14569b1bcb6694087cf5b215d87f0e93121e57d0a31340b2632e66726` reports.
There are 161 fixture outcomes (39 accepted, 122 rejected), four immutable
inputs, and 36 newly verified assertion IDs. [The index](evidence/provenance-index.json)
binds the code, policy, snapshot, input copies, unchanged original assertion
body/helpers, compiler, and test binary. This is document parity only. The
preceding whole-lock canonical/archive checkpoint has its own earlier SHA.

```sh
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_PROVENANCE_REPORT=/absolute/evidence/provenance.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_provenance_documents -- --ignored
python3 scripts/rc9-provenance-policies /absolute/snapshots documents /absolute/new-provenance.toml
python3 scripts/rc9-provenance-policies /absolute/snapshots lock /absolute/new-lock-inventory.toml
```

Use a fresh report path for each run. Generated fragments remain partial
bundles; the lock fragment now has its separate frozen differential report below.

## Whole-lock differential and resolver regression qualification

At `5251b4f056741768eb667c74d6e0949257ca843b`, `scripts/check` passed structure,
formatting, strict lint, 1,518 test outcomes (zero failures, 9 explicitly
ignored qualification/advisory tests), and rustdoc. Self-analysis is complete:
965 Rust files, 1,540 base contexts, no derived contexts, 1,114,966 projection
work, and no unresolved items. It exits 1 at exactly `LOCK-008`, `LOCK-026`,
`LOCK-028`, and `LOCK-030`. The reviewed rc8 lock remains unchanged.
Standalone `scripts/package-check` passes on that same clean revision; the
canonical gate therefore remains interrupted rather than being reported green.

The same revision passed two byte-identical whole-lock differential runs:
5 policies, 12 bound inputs, 50 fixtures (20 accepted, 30 rejected). Payload
SHA-256: `ac73042f419004282fa1fb0e93c03a9c03b465933bdb12460c0551dcba70ffca`. The [index](evidence/lock-parity-index.json)
binds every log, source/policy identity, original assertion body, and outcome.

```sh
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_LOCK_PACKAGES_REPORT=/absolute/evidence/whole-lock.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_lock_inventories -- --ignored
scripts/check
scripts/package-check
```

Use fresh output paths. The index also preserves two failed intermediate
canonical runs: `43b1ddc37932b5e48ddc878399572287b6a4f7e9` exceeded the unchanged
1,232,039 work ceiling with 1,233,366 queries, and
`365406305a9731c8feae86fc920b63c42fd553d6` exposed two unapproved JSON macro uses.
Completed lexical-boundary caching repaired the workload; typed report records
removed the macro violations. No ceiling or macro authority was broadened.
The 141 focused resolver tests and strict workspace lint also pass.

## Raw dependency differential qualification

Two clean runs at `58e5835c82915144ebd1f147e536a9cbcb0af170` produced
byte-identical reports for 16 policies and four frozen inputs: 157 fixture
outcomes, 85 accepted and 72 rejected. Payload SHA-256:
`81ec3aaece0c9952bca02d517c5a016cfe22d08a9aa78c44270f0d7a66715194`. The [index](evidence/raw-dependency-index.json)
binds the source, policy, fixture origins, compiler, test binary, and logs.

```sh
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_RAW_DEPENDENCY_REPORT=/absolute/evidence/raw-dependency.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_raw_dependency_assertions -- --ignored
python3 scripts/rc9-raw-dependency-policies /absolute/snapshots /absolute/new-raw-policy.toml
```

Structure, format, strict workspace lint, the focused suite, and both frozen
runs pass. The same committed tree has complete self-analysis:
Analysis: complete; 969 Rust files, 1,544 base contexts, 0 derived contexts, 1,115,586 projection work, 0 unresolved.
Self-check exits 1 with exactly the four existing rc8 lock-drift diagnostics.
The complete canonical/archive checkpoint remains the earlier `5251b4f`; this
slice does not claim that gate was rerun on its newer revision.

## Qualification text and raw structure

At `7c6c01d0af8bdeee598e87c3c8fae97ce72b9a0c`, two frozen qualification-text runs
produce byte-identical reports: 125 policies, 24 inputs, and 923 fixtures
(410 accepted, 513 rejected). Payload SHA-256:
`2f5a621fae8b041c719bb65f42948808a2eee111e358ba9f9c4717263564ab15`.
The [index](evidence/qualification-parity-index.json) binds source/policy identities,
compiler/test binary, exact extracted bodies, fixture copies, and every log.

```sh
python3 scripts/rc9-qualification-policies /absolute/snapshots /absolute/new-qualification.toml
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
export ZRAIL_RC9_QUALIFICATION_REPORT=/absolute/evidence/qualification-a.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_qualification_assertions -- --ignored
export ZRAIL_RC9_QUALIFICATION_REPORT=/absolute/evidence/qualification-b.json
cargo test --locked --offline -p zrail-rust \
  qualify_all_frozen_kafka_driver_qualification_assertions -- --ignored
cmp /absolute/evidence/qualification-a.json /absolute/evidence/qualification-b.json
scripts/check
scripts/package-check
python3 scripts/rc9-inventory-check
python3 scripts/rc9_qualification_evidence_test.py
```

Use fresh report paths in the same evidence directory, with a clean checkout at
the stated implementation commit to reproduce that evidence. The ledger verifier
checks exact policy selectors and values, every required case and diagnostic,
all 24 bound inputs, and the absence of unrelated input changes. Seven deliberate
evidence mutations were rejected: predicate, selector, missing case, duplicate
case, unrelated input digest, diagnostic category, and overstated analysis claim.

On that same revision, `scripts/check` passes structure, format, strict lint,
**1,531 test outcomes** (zero failures, 11 explicitly ignored qualification/advisory
tests), and rustdoc. Self-analysis is complete: 987 Rust files, 1,566 base contexts,
zero derived contexts, 1,121,116 projection work, and zero unresolved items.
It exits 1 at exactly `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030` because
the reviewed rc8 lock remains unchanged. Standalone `scripts/package-check`
passes all three normalized archives and their offline expanded-archive checks.
Git status remains empty. Those separate checks do not turn the interrupted
canonical gate into a pass, and no lock authority was accepted.

The engine checkpoint `6d638888e5d784dd3448842cf04ab05421ca7e04` additionally
records 1,530 passing canonical test outcomes and the same four lock diagnostics;
its archive stage was not run separately. New predicate tests cover schema limits,
protected weakening, first-occurrence byte offsets, interval boundaries, complete
line counts with bounded escaped samples, invalid UTF-8, and stale input binding.
The new raw claims preserve comments and strings deliberately; they do not prove
workflow structure, Rust code order, or command execution. Final versioned-tree
release qualification and full downstream policy bundles remain open.

## Authored method inventory checkpoint

At `e5f8178123658369ba8cbfbbc0ad068261be03c9`, `scripts/check` passes structure,
formatting, strict workspace lint, **1,548 tests**, and rustdoc, then stops at
exactly `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030`. Source analysis is
complete: 1,005 physical Rust files, 1,594 base contexts, zero derived contexts,
1,166,682 projection work, and zero unresolved items. Eleven qualification/timing
tests require separate trusted invocations and remain ignored in the ordinary
gate. Standalone `scripts/package-check` passes all three normalized archives and
offline expanded checks at the same revision; Git status remains empty. The
canonical archive and cleanliness steps are not reached, and the gate remains
failed pending protected lock review.

The [machine-readable checkpoint](evidence/method-inventory-index.json) binds
both canonical logs and the archive log. The earlier `523bfd1` canonical run
identified five additional self-hosting diagnostics. They were corrected by
splitting the public export facade, placing the authored index under the parser,
sharing the explanation's macro policy reference, and moving test-only process
identity to its actual test file. Neither the reviewed contract nor lock changed.

```sh
cargo test --locked --offline -p zrail-core inventories
cargo test --locked --offline -p zrail-testkit --test rust_inventories
cargo test --locked --offline -p zrail-rust frozen_transport_methods_match
python3 scripts/rc9-snapshots /absolute/snapshot-directory
python3 scripts/rc9-transport-method-policies /absolute/snapshot-directory /fresh/policy.toml
scripts/check
scripts/package-check
```

Six schema/diff tests, ten integration tests, and twenty synthetic syntax cases
exercise quantities, exact maps, receiver-independent spelling, qualified const
generics, attributes, macro boundaries, cfg branches, repeated includes, excluded
sources, missing subjects, malformed files, expression-only fragments, and bound
input changes. The complete original transport collector is extracted byte for
byte, with its parser and expected map, and compared against native observations.
An initial mismatch demonstrated why macro-token observations cannot substitute
for authored AST membership. Existing macro-aware facts remain unchanged; the
optional authored index reuses ordinary facts and supplies otherwise unvisited
expression contexts from the same parse.

The translated method fragment preserves 11 `(file, method)` entries totaling
20 occurrences. Its SHA-256 is
`499efcbedcf45bdafbd2aa57a1383cdb91cb7680da4befdc503a4f75443fb027`.
It remains a partial policy. Complete frozen production selection and the exact
original detector fixture still require differential qualification. No new
assertion is marked verified at this checkpoint.


## Expanded fallible-call census

[The census checkpoint](evidence/census-fallible-index.json) records two successful
offline runs at `e01891c5843757f9f9cb5e1f5f39cfc21ca6e583`, with identical SHA-256
`f90b74831614e4a660fdc8d1c008eb9f036fd81350f931478322af1378cab161`. It adds 7,794 candidates while
preserving every prior candidate and frozen file identity. It does not increase
verified replacement coverage. The existing 64 MiB artifact bound is retained,
and the producer requires a fresh output path.

```sh
cargo test --locked --offline -p zrail-rust --test rc9_inventory
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots ZRAIL_RC9_CENSUS=/fresh/census-a.json \
  cargo test --locked --offline -p zrail-rust --test rc9_inventory write_frozen_assertion_census -- --ignored
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots ZRAIL_RC9_CENSUS=/fresh/census-b.json \
  cargo test --locked --offline -p zrail-rust --test rc9_inventory write_frozen_assertion_census -- --ignored
cmp /fresh/census-a.json /fresh/census-b.json
python3 scripts/rc9-inventory-check --census /fresh/census-a.json
```


## Frozen transport method and parser qualification

The [final transport index](evidence/transport-methods-index.json) binds clean
revision `151270ffa2285800833f2295abcfc61aa689d9ca`, tree
`cc0beffaef20c26cc0c292b9dd6a3cde0f86ffe9`, compiler, test binary, Cargo lock,
policy, original registry/collector/fixture identities, and every source input.
The untouched kafka-driver snapshot is
`a45be8071e6a663cd4ae3142cc5304ece9fdf45e`. Both offline runs passed all
76 fixtures (seven accepts, 69 rejects), with identical payload SHA-256
`431fb863939196180696673b66350ab81538867595e35f8b1d6d846e55d15bf7`.

At that same revision, `scripts/check` passed structure, formatting, strict
workspace lint, **1,551 tests** (zero failures, twelve ignored), and rustdoc.
Self-analysis completed with 1,010 Rust files, 1,599 base contexts, zero derived
contexts, 1,168,340 projection work, and zero unresolved items. The canonical
gate **failed** with only `LOCK-008`, `LOCK-026`, `LOCK-028`, and `LOCK-030`;
its archive and cleanliness steps were not reached. Separately invoked
`scripts/package-check` passed all three normalized archives and offline expanded
checks, and Git status remained empty. No reviewed authority was accepted.

The index retains earlier qualification failures under their own revisions:
the initial comparison used a different path sort order for an identical file
set; subsequent self-checks rejected computed/literal inclusion of excluded
fixture source and unreviewed `eprintln`; the first runtime-reader revision
stopped at formatting. The final reader hashes frozen bytes at runtime in the
trusted test harness, and the corrected revision was completely rerun.

```sh
# Use a clean checkout at the pinned implementation revision and prefetched snapshots.
scripts/check
scripts/package-check
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_METHODS_REPORT=/fresh/transport-a.json \
  cargo test --locked --offline -p zrail-rust qualify_all_frozen_kafka_driver_transport_methods -- --ignored
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_METHODS_REPORT=/fresh/transport-b.json \
  cargo test --locked --offline -p zrail-rust qualify_all_frozen_kafka_driver_transport_methods -- --ignored
cmp /fresh/transport-a.json /fresh/transport-b.json
# Run these binding checks from the later evidence-bearing checkout.
python3 scripts/rc9_method_evidence_test.py
python3 scripts/rc9-inventory-check
```

The artifact-only validator independently checks the typed mutation matrix,
full count maps, original parser outcomes, frozen input identities, policy
selectors, quality, complete totals, and intended diagnostics. Its two tests
include twenty evidence-tampering variants. This closes four assertion IDs,
without claiming full consumer Cargo/source, lock, or execution qualification.


## Expression-path implementation checkpoint

At clean `b763cebc431f6f2449c32025e88a68323d08da63`, the canonical gate passed
structure, formatting, strict workspace lint, the assertion ledger and four
artifact-validator tests, **1,557 Rust tests** (zero failures, twelve ignored),
and rustdoc. Source analysis completed over 1,015 Rust files and 1,606 base
contexts with 1,172,598 projection work, zero derived contexts and zero unresolved
items. [The checkpoint index](evidence/expression-inventory-index.json) binds
that revision, tree, policy and both logs. The gate remains **failed** on
`LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028` and `LOCK-030`.

`LOCK-016` is the reviewed check-runner digest changing when the new parity
evidence validation was added to `scripts/check`. The stronger gate is retained;
no root contract or lock was changed and no authority was accepted. The trusted
runner initially flagged this extra diagnostic for inspection; the source-bound
gate digest explains it. Standalone archive verification passed all three
normalized archives and expanded offline checks, with a clean pinned checkout.
The canonical archive and cleanliness steps were not reached.

Eight core schema/diff tests, thirteen source integration tests and twenty-two
frozen expression-path syntax comparisons pass. The partial expression-path
policy SHA-256 is
`4eff9ae08134fb802fcd4abffdc7bfc63ce79f03d40f648ee44afb6d76fe7a58`.
Full frozen-source and original detector qualification remain pending at this
checkpoint; no further assertion is marked verified.


## Explicit Rust fragment census

[The fragment census index](evidence/census-fragments-index.json) binds clean
`951941ff2d8964ac8a6f61aef2e6bd7f5403cef7`, tree
`9e6bccbf6a7da51a88f520adf6c708a3eb7fc397`, the seven-input registry, compiler,
test binary and both successful offline logs. Five focused census tests and
strict census-target lint passed. The two regenerated payloads are byte-identical
and preserve every prior candidate object and frozen file identity.

```sh
cargo test --locked --offline -p zrail-rust --test rc9_inventory
cargo clippy --locked --offline -p zrail-rust --test rc9_inventory -- -D warnings
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots ZRAIL_RC9_CENSUS=/fresh/fragments-a.json \
  cargo test --locked --offline -p zrail-rust --test rc9_inventory write_frozen_assertion_census -- --ignored
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots ZRAIL_RC9_CENSUS=/fresh/fragments-b.json \
  cargo test --locked --offline -p zrail-rust --test rc9_inventory write_frozen_assertion_census -- --ignored
cmp /fresh/fragments-a.json /fresh/fragments-b.json
# Run from the later evidence-bearing checkout.
python3 scripts/rc9-inventory-check --census /fresh/fragments-a.json
```

The inventory validator requires the exact fragment-registry digest, source
revisions/hashes, reviewed including/fixture source hashes and complete registry
counts. Two artifact-validator tests include ten rebound-registry tampering
cases; these tests are also part of `scripts/check`. The census contains 80,135 candidates; its new SHA-256 is
`308d9fcb3b237d706f35b51069cb336b0ed6dd225f589af22e5c05554749a7c5`.
The discovery fix adds 186 unreviewed candidates and closes no replacement
assertion. Canonical final-tree and complete consumer qualification remain open.


## Frozen expression-path differential qualification

[The expression-path index](evidence/transport-expression-paths-index.json) binds
`f2db6e496f763224b2ffeec41dd0fa1450b4b80f`, tree
`5806fdef6f45fadc066b281b8d8304ca980ec5cf`, compiler, binary, Cargo lock, policy,
original registry/collector/assertion/fixture identities and all source inputs.
Both offline runs passed 83 fixtures (27 accepts, 56 rejects) and produced
identical payload SHA-256
`126e64df5112d09775d6542392da6bb3883f75a7cb65cffbab88e1fad67a702f`.
This closes the two expression-path assertion IDs with intended native
`RUST-INVENTORY-001` diagnostics and no unexplained protection differences.

At that same revision, `scripts/check` passed structure, formatting, strict
workspace lint, artifact integrity and four Python validator tests, **1,558 Rust
tests** (zero failures, thirteen ignored), and rustdoc. Source analysis completed
over 1,018 Rust files and 1,609 base contexts with 1,173,947 projection work, zero
derived contexts and zero unresolved items. The gate **failed** only on the five
inspected lock diagnostics: `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028` and
`LOCK-030`. Standalone verification passed all three normalized archives and
expanded offline checks, with clean Git status. Canonical archive/cleanliness
steps were not reached and no authority was accepted.

The initial `d2b2772` gate also rejected eight `CAP-001` occurrences because the
shared trusted runner had been moved beneath analyzer source. The corrected
revision places process execution in `tests/rc9_transport`; it preserves the
existing non-execution boundary without a policy grant. Both logs retain their
own code identities. At `756d51d`, a human diagnostic wording fix distinguishes
expression paths from method calls; thirteen integration tests passed, and the
report's existing count/claim/diagnostic-ID evidence remains accurately scoped
to its earlier revision. Final versioned-tree qualification is still required.

```sh
# Clean checkout at the indexed implementation revision; prefetched snapshots.
scripts/check
scripts/package-check
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_EXPRESSION_PATHS_REPORT=/fresh/expression-a.json \
  cargo test --locked --offline -p zrail-rust qualify_all_frozen_kafka_driver_transport_expression_paths -- --ignored
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_EXPRESSION_PATHS_REPORT=/fresh/expression-b.json \
  cargo test --locked --offline -p zrail-rust qualify_all_frozen_kafka_driver_transport_expression_paths -- --ignored
cmp /fresh/expression-a.json /fresh/expression-b.json
# Later evidence-bearing checkout: four tests, forty tampering cases.
python3 scripts/rc9_method_evidence_test.py
python3 scripts/rc9-inventory-check
```

## Frozen exact-owner qualification

The [owner evidence index](evidence/transport-owners-index.json) binds revision
`faa606e33df78d4fb676338ac0c80cfb7df8d5ad`, tree
`b17fd12ac90b9b387c75b37d8da6eb882514b1c2`, with the exact frozen Kafka-driver
snapshot, original collector/assertions, translated policy, compiler and test
binary. Both offline runs passed all 46 cases (15 accepts, 31 rejects) with
identical 4,737,979-byte payloads, SHA-256
`623558e93ecb1999fdb2298d7ae477fb81836aa117144eb6f49b7f9cb275a370`.

The selected universe is 479 physical production-path files from 772 bound Rust
inputs. The original assertion requires one owner file; native evidence also
records its six authored path occurrences. `legacy_measure = "distinct-owner-files"`
means each legacy map entry denotes one membership, independently of native
occurrence quantities. Duplicate occurrences preserve owner membership; missing,
relocated, substituted or additional owner files fail. The matrix also covers
26 syntax contexts, the unchanged original detector, all six roots, path-based
test exclusion, malformed files and expression-only fragments. The artifact
validator independently checks actual quantities, all inputs and the complete
mutation matrix; seven unittest methods reject 64 evidence mutations across the
three verified transport families.

At that same revision, `scripts/check` passed structure, artifact checks,
formatting, strict workspace lint, 1,573 Rust tests (zero failures, 14 explicitly
ignored), and rustdoc. Self-analysis completed with 1,024 Rust files, 1,615 base
contexts, 1,178,356 projection work, and zero unresolved observations. The gate
failed only on `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`, and `LOCK-030`.
Its archive/cleanliness steps were not reached; separate `scripts/package-check`
and explicit Git cleanliness checks passed. No authority was accepted. The
initial `5efa1e2` Clippy failure remains recorded under its own revision.

```sh
# Use the pinned clean implementation checkout and prefetched frozen snapshots.
scripts/check
scripts/package-check
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_OWNERS_REPORT=/fresh/owners-a.json \
  cargo test --locked --offline -p zrail-rust --lib \
  qualify_all_frozen_kafka_driver_transport_owners -- --ignored
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_OWNERS_REPORT=/fresh/owners-b.json \
  cargo test --locked --offline -p zrail-rust --lib \
  qualify_all_frozen_kafka_driver_transport_owners -- --ignored
cmp /fresh/owners-a.json /fresh/owners-b.json
# Run these artifact checks in the later evidence commit containing the reports.
python3 scripts/rc9_method_evidence_test.py
python3 scripts/rc9-inventory-check
```

This verifies `KD-TRANSPORT-OWNERS` and `KD-TRANSPORT-DETECTOR-OWNERS` only.
The import-rename capability at `b340af6` has targeted tests and strict workspace
lint; it is not covered by this earlier checkpoint. Complete downstream policy
bundles, Cargo/source completeness, final-release revision qualification and
remaining transport assertions remain open.

## Import-rename qualification

The clean isolated checkpoint `405fe5674f77b8a31bd5aea01b190da55c7dcbcf`, tree
`249a9080f851eee3579187eed463e3f46ab6cbaf`, passed 1,580 Rust tests (15 explicitly
ignored), artifact validation, seven Python unittest methods, formatting, Clippy
and rustdoc. `scripts/check` stopped at the same five pending protected-lock
diagnostics: `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`, `LOCK-030`.
Self-analysis was complete: 1,030 files, 1,622 contexts, 1,182,150 work, zero
unresolved. Standalone `scripts/package-check` passed and Git status stayed clean.
This is not a passing canonical release gate; no lock authority was accepted.

With the environment above and a clean checkout at that exact commit, run:

```sh
ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots \
ZRAIL_RC9_TRANSPORT_RENAMES_REPORT=/absolute/evidence/rename-a.json \
cargo test --locked --offline -p zrail-rust --lib \
  qualify_all_frozen_kafka_driver_transport_renames -- --ignored
# Repeat with rename-b.json in the same evidence directory, then compare bytes.
cmp /absolute/evidence/rename-a.json /absolute/evidence/rename-b.json
python3 scripts/rc9_method_evidence_test.py
python3 scripts/rc9-inventory-check
```

Both differential runs passed 79 cases (23 accepted, 56 rejected) with identical
8,000,231-byte payloads. The [index](evidence/transport-renames-index.json) binds
all reports, logs and exact identities. The evidence validates authored syntax
over the complete frozen detector selection; it does not certify Cargo resolution,
full downstream policy, execution receipts, or removal of the old scanner.
The later artifact validator passes ten unittest methods and 89 adversarial
mutations; that later validation is distinct from the pinned checkpoint's tests.

## Resumed trait-set qualification

The [trait index](evidence/transport-impls-index.json) binds two complete fresh
runs at `53936654ffbaff1bb0e9e127db46d3ca0aa510ba`, tree
`867f379edce0e69867392247bf569d76e5f5038c`. Each passes 78 cases: 28 accepted,
50 rejected. Both payloads contain 8,058,906 bytes with SHA-256
`a6287bc87a993c2d782cfc24ccfb3e21dbb866dd0c697c0cb8e17da74547c73e`.
All 772 frozen Rust inputs, 479 selected physical files, the original detector,
registry, policy and compiler inputs are bound. The only change from the stopped
first report is the fresh absolute fixture directory. Stopped artifacts remain.

```sh
# Clean isolated checkout at the indexed revision, after trusted prefetch.
export ZRAIL_RC9_SNAPSHOTS=/absolute/snapshots
# Create a fresh external directory; use the same directory for both reports.
ZRAIL_RC9_TRANSPORT_IMPLS_REPORT=/fresh/trait-a.json \
  cargo test --locked --offline -p zrail-rust --lib \
  qualify_all_frozen_kafka_driver_transport_impls -- --ignored
ZRAIL_RC9_TRANSPORT_IMPLS_REPORT=/fresh/trait-b.json \
  cargo test --locked --offline -p zrail-rust --lib \
  qualify_all_frozen_kafka_driver_transport_impls -- --ignored
cmp /fresh/trait-a.json /fresh/trait-b.json
# Evidence-bearing checkout: verify strict bindings and tampering rejection.
python3 -B scripts/rc9_method_evidence_test.py
python3 -B scripts/rc9-inventory-check
```

The validator suite has 13 methods and 116 adversarial mutations, including
trait substitution, implementing-type substitution, broadened type filters,
duplicate quantities, membership substitution and forged provenance. The two
trait assertions are verified within their authored-syntax scope. The indexed
canonical run at this revision passed 1,591 Rust tests, with 16 ignored, and
completed analysis of 1,038 files / 1,632 base contexts with zero unresolved.
It failed `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028` and `LOCK-030`.
Its standalone archive check passed; no authority was accepted.

## Goal resumption working-tree qualification

The resumed mixed route-policy slice passed three focused tests (all ten source
files, raw matching contexts, exact facade count, required input and UTF-8
mutations), formatting, structure and strict Clippy. All 28 census candidates
in the mixed original file are linked to 31 assertion instances; full route
qualification and runtime execution evidence remain pending.

A full `scripts/check` run at HEAD `01bbf38481217b0611a387ee2af488328f52137c`
plus the recorded working changes passed 1,602 Rust tests (zero failures,
16 ignored), artifact validation, formatting, Clippy and rustdoc. Self-analysis
was complete: 1,048 files, 1,643 base contexts, 1,191,665 work, zero unresolved.
The same five protected-lock diagnostics stopped the canonical gate. This is
working-tree evidence, not qualification of a new committed release revision.
The external `rc9-goal-resume-check-20260907/runner.json` binds its tracked patch
and all untracked input digests and confirms those inputs did not change during
the run. Standalone `scripts/package-check` subsequently passed; its result does
not turn the failed canonical gate into a pass. No lock or root contract was changed.
