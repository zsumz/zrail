# rc9 guardrail replacement qualification

Release verdict: **blocked**. Inventory review and replacement qualification are
incomplete. No consumer guard has been deleted and no release artifact has been
published. The user approved retiring Rafter's private-name policy; see
[policy decisions](DECISIONS.md).

This branch implements the strict facade slice. It does **not** yet deliver rc9
replacement coverage. See [exact reviewed assertion blockers](BLOCKERS.md),
[qualification commands and results](QUALIFICATION.md), and
[conditional downstream cutovers](CUTOVERS.md).

## Immutable inputs

The machine-readable [snapshot manifest](../../crates/zrail-testkit/tests/fixtures/rc9/snapshots.json)
binds repository, commit, and Git tree. All four supplied commits were resolved
and verified against fetched Git objects on 2026-09-05.

| Repository | Frozen commit | Governed workspaces to review |
| --- | --- | --- |
| zsumz/zrail | `5a368379360104ca19745326cfcef48d22a6452b` | root |
| kafkars/kafka-driver | `a45be8071e6a663cd4ae3142cc5304ece9fdf45e` | root |
| kafkars/kafkars | `67ad7aa8d91f9639c87b00338b9e21a633f7bb4c` | root; detector fixture workspaces are test inputs |
| zsumz/rafter | `e99b43c1b3fb3c94dd626effccee8420016594f2` | root, `reference`, `bench-compare`, `fuzz` |

Implementation starts at exactly the audited rc8 commit and tree, with **zero
differences from rc8**. Work uses branch `feat/rc9-guardrail-coverage` in an
isolated worktree. The pre-existing rc7 feature checkout remains untouched.
That checkout's `a56421bb1ef26c0b0756762421d17f684e03a1a1` is not the audited
base: rc8 also includes external macro export/cache work and source-staging fixes.
It must not be used as a substitute for rc8 in compatibility qualification.

Trusted setup, separate from runtime analysis:

```sh
python3 scripts/rc9-snapshots /absolute/snapshot-directory --prefetch
python3 scripts/rc9-snapshots /absolute/snapshot-directory
```

The second command is offline and rejects wrong revisions, wrong trees, and
dirty snapshots. Setup never resets existing source. Qualification outputs and
mutated fixtures belong outside these frozen checkouts.

## Assertion review rules

An entrypoint is not an assertion. Each live assertion, each independently
failing helper predicate, and each instantiated registry requirement needs its
own migration identity. Detector tests link to the predicates they exercise;
they do not prove that an entire mixed source file can be retained or removed.
Implementation and verification are separate states. No disposition is verified
until its intended policy diagnostic has positive and negative evidence.

Supported dispositions are `existing native rail`, `declarative translation`,
`new engine capability`, `behavioral evidence retained`, and `approved retirement`.
The last disposition requires an external policy decision. An unreviewed source
surface remains an inventory blocker, not an inferred behavioral exemption.

## Machine-readable audit state

The [assertion ledger](../../crates/zrail-testkit/tests/fixtures/rc9/assertions.json)
contains **103 reviewed assertion instances**. The
[full tracked-file census](../../crates/zrail-testkit/tests/fixtures/rc9/census.json.gz)
and [summary](../../crates/zrail-testkit/tests/fixtures/rc9/census-summary.json)
record 9,792 files and 72,155 syntax candidates. Every tracked path is included,
including nested workspaces, sources outside `src/`, helper code, and malformed
detector fixtures. Non-Rust file contents, helper expansion, registries, and
opaque syntax still require review; this is not a completed assertion inventory.

| Repository | Tracked files | Rust files | Reviewed assertion instances | Implemented predicates | Verified detector assertions |
| --- | ---: | ---: | ---: | ---: | ---: |
| kafka-driver | 826 | 772 | 78 | 3 | 2 |
| Kafkars | 6,808 | 6,713 | 10 | 4 | 3 |
| Rafter | 2,158 | 1,897 | 15 | 0 | 0 |

The five verified assertions are `KD-FACADE-FUNCTION`, `KD-FACADE-INLINE`,
`KF-FACADE-FUNCTION`, `KF-FACADE-INLINE`, and `KF-FACADE-IMPORT`. Their disposition
is **new engine capability**. Verified counts for every other disposition are
zero. They prove detector predicate/diagnostic parity, not a live repository's
selection or complete policy bundle. `KD-FACADE-LIVE` and `KF-FACADE-LIVE` remain
unverified. No full repository has replacement qualification.

The uncompressed census SHA-256 is
`69c6393f176df64fff2e010968f18efb4e06781385b1bd6896de013411899ba6`.
Two independent runs produced identical bytes. Candidate IDs bind repository,
physical path, source coordinates, candidate kind, and normalized syntax digest.
The one parse boundary is the deliberately invalid Kafkars invariant-registry
fixture; it has not been excluded or treated as a passed assertion.

`python3 scripts/rc9-inventory-check --require-complete` currently fails. Passing
the artifact integrity check without that flag certifies only consistent data.

## Implemented facade slice and compatibility

- `wiring-only` accepts external module declarations and all Rust `use` items,
  matching kafka-driver's frozen predicate.
- `wiring-reexports` accepts external modules and public or crate/super-rooted
  restricted re-exports, matching Kafkars' frozen predicate.
- Both strict modes reject functions, impls, inline modules, data declarations,
  extern-crate declarations, and opaque item macros. Existing `declarative` and
  `allow` contracts preserve rc8 behavior and observations.
- Exact, reasoned `test-facade` declarations impose structure independently of
  compilation reachability, test identities, placement, and current test budgets.
  A production mount or missing test mount fails its intended role rule.
- Explain output exposes the effective mode and reason. Coverage schema 6 lists
  selected facade policies and every written violation without truncating totals.
- Mode weakening and removed or redirected facade authority remain protected.
  Analyzer semantics advance from 6 to 7; lock schema remains 3. Existing
  migration paths are preserved, including the newly supported rc8 path.

The frozen predicates are extracted byte-for-byte into the trusted Rust test
suite. Fifty-six differential comparisons and eight integration acceptance
tests pass. The imported Kafkars negative fixture is also byte-identical to its
source. This is the only implemented new policy family in this branch.

Strict same-revision rc8 reanalysis preserves all 36 compared authority entries.
The reviewed root lock has not been replaced or accepted. The final self-check
has four lock/input-drift diagnostics; the ordinary diff reports two protected
epoch-comparison unknowns. See the archived migration preview and qualification
report before proposing any authority update.

## Findings that constrain the implementation

- kafka-driver's `transport_authority.rs` counts `ExprPath` occurrences, including
  function-value acquisition. Its five top-level inventory comparisons have
  different quantities: owner-file set, per-file associated-path counts, authored
  rename set, per-file written method counts, and type/trait implementation set.
  Existing direct-call owners alone cannot replace these assertions.
- kafka-driver's facade detector accepts every `use` item and external module
  declarations. Kafkars accepts external modules and public or crate/super-rooted
  restricted re-exports, but rejects private imports and `pub(self)` imports.
- Kafkars' Fetch preparation guard has three ordered sequences and separate
  presence requirements. Source ordering must not be reclassified as behavior or
  replaced by an owner-file allowlist.
- Kafkars requires an exact measured baseline above target and a separate
  reason/owner/issue allowance above hard. Rafter reports target/soft excess as
  warnings and fails only above hard without a reviewed tracking allowance.
  These have different acceptance semantics.
- Rafter's root size guard scans `crates` and `fuzz`. It does not scan
  `bench-compare` or `reference`; those separately governed workspaces require
  their own inventory and invocation. The reference size evaluator is embedded
  in an actual build/test/doc runner, which remains intact.
- Rafter's readability guard intentionally permits data declarations in its
  facades. Applying wiring-only policy there would be a policy change.
- kafka-driver prohibits the text `zrail` in its current CI workflow. Its proposed
  cutover must explicitly replace this assertion with an enforced architecture
  lane; merely installing the binary cannot satisfy the replacement mission.
- Rafter's final wire-tag violation assertion combines `.u8(...)` argument-shape
  checks with three independent raw, whitespace-compacted alias prohibitions.
  These are separate `RF-WIRE-U8-SHAPES` and `RF-WIRE-ALIAS-*` entries.
- Rafter's raw-process guard requires 18 exact file/context/path/count tuples,
  authored canonical spelling, and separate crate-alias and macro-token bans.
  The complete expected tuple map is recorded under `RF-PROCESS-EXACT`.
- Rafter's private-name scan obtains its patterns outside the repository. The
  user approved retiring this policy in `RC9-DECISION-RF-PRIVATE-NAMES`; supplying
  those patterns is no longer a release prerequisite. Independent package
  construction and artifact verification remain required.
- A normal parsed module-doc predicate does not preserve kafka-driver's literal
  leading `//!` requirement. `KD-MODULE-LEADING-DOC` records that gap explicitly.
- Kafkars traversal has required-directory and minimum-file-count assertions in
  shared helpers. Reviewing only the named facade or ownership tests would miss
  these independently failing requirements.

## Open release blockers

| ID | Cause |
| --- | --- |
| RC9-INVENTORY-KD | Complete assertion/helper/registry/fixture/CI review for kafka-driver, including smoke-policy and packaging surfaces. |
| RC9-INVENTORY-KF | Complete assertion-level review of the Kafkars guardrail crate and source outside it. |
| RC9-INVENTORY-RF | Complete assertion-level review of Rafter, its independent evidence tooling, and separate workspaces. |
| RC9-POLICY | Complete stock-engine policy bundles, with no unsupported generic predicates. |
| RC9-PARITY | Execute frozen legacy and replacement detectors with intended-diagnostic assertions and account for every difference. |
| RC9-FULL | Complete source discovery, checks, coverage, receipts, and cutover maps on every governed workspace. |
| RC9-RELEASE | Complete compatibility, protected semantic diff, final versioned-tree qualification, and release documentation. |

These are discovery/release blockers, not a claim that assertion inventory is
complete. [BLOCKERS.md](BLOCKERS.md) lists all **98 reviewed, unverified assertion
IDs and their causes**. The remaining candidate IDs are explicitly enumerated in
the census and need review before their disposition or complete assertion
expansion can be known. No unsupported generic assertion has been declared
behavioral merely to remove it from the release criterion.
