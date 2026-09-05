# rc9 guardrail replacement qualification

Release verdict: **blocked**. Inventory review and replacement qualification are
in progress. No consumer guard has been deleted, no policy retirement has been
approved, and no release artifact has been published.

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
- Rafter's root size guard includes `fuzz` and `bench-compare`; the separately
  governed reference workspace has its own size evaluator embedded in an actual
  build/test/doc runner. Replacing that evaluator must preserve the runner.
- Rafter's readability guard intentionally permits data declarations in its
  facades. Applying wiring-only policy there would be a policy change.
- kafka-driver prohibits the text `zrail` in its current CI workflow. Its proposed
  cutover must explicitly replace this assertion with an enforced architecture
  lane; merely installing the binary cannot satisfy the replacement mission.

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
complete. Assertion IDs will refine them as review proceeds. Verified disposition
counts are currently zero for every disposition.
