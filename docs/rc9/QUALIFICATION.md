# Reproducing the current rc9 evidence

Release verdict: **blocked**. The latest size-parity implementation is commit
`767abfd0f4f26c97fa48433c300fce999f567778`, based on the exact audited rc8 commit.
Later audit/report-only changes do not constitute final versioned-tree release
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
`71485d30104be0e77cb21563ea0ff28a36f6ccbb0f665e7f9f3d739abd31391b`.
The implementation was clean when this report ran. Kafkars' and Rafter's
unmodified frozen size targets additionally passed five tests each; receipts
record their original source context, pinned compiler dependencies and outcomes.
The private-name retirement did not modify any frozen input.

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
