# Reproducing the current rc9 evidence

Release verdict: **blocked**. The latest metadata parity, canonical gate, and
archive checkpoint is `db11ebe40f3ffe969939785b34f2acf880c6b79e`, based on the
exact audited rc8 commit. Earlier facade, size, and file evidence retains its
own implementation identity below. None constitutes final versioned-tree release
qualification. The workspace version and internal pins remain `0.0.3-rc.8`.

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
| Repeated all-file census | **Pass**, 2 tests, byte-identical repeated JSON. 9,792 tracked files and 72,155 syntax candidates. Discovery is not completed assertion review. |
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
