# Stopped rc9 work

Work stopped at the user's request. All running zrail qualification, Cargo and
test processes were terminated and the host process list was checked afterward.
No additional implementation or qualification is authorized by this checkpoint.

The implementation head before this checkpoint is
`b800c03e00664dba51737da9ab5ee63ead0ab449` on
`feat/rc9-guardrail-coverage`, in `/root/zrail-rc9`. The original `/root/zrail`
worktree and reviewed root `zrail.toml`/`zrail.lock` remain unchanged.
No downstream guards were removed. No push, tag or publication occurred.

The ledger remains **575 reviewed, 508 implemented, 494 verified**, with no
complete downstream repository qualification. Release verdict: **blocked**.
See [the parity report](PARITY.md) and [remaining assertion IDs](BLOCKERS.md).

## Saved work

- `163d3d1` implements written trait/type inventories; `7aaa115` corrects the
  test-file mounting; `813bbdc` adds the exact frozen trait-set policy and runner.
- `5393665` records verified import-rename evidence: 79 cases, 23 accepted and
  56 rejected, with two byte-identical reports. Its artifact validation passed
  ten unittest methods and 89 adversarial mutations.
- `b800c03` implements direct file-module and enum-variant inventories. Targeted
  validation passed 23 core inventory tests, 28 integration tests, 36 frozen
  declaration cases, original-expression AST equivalence, structure and Clippy.
  Declaration-policy translation and full-snapshot evidence remain unfinished.
- This checkpoint saves the in-progress trait evidence validator and the first
  completed report. Only Python syntax was checked after the stop request;
  validator behavior and tamper tests are not verified.

## Interrupted qualification

The isolated checkpoint at `53936654ffbaff1bb0e9e127db46d3ca0aa510ba`, tree
`867f379edce0e69867392247bf569d76e5f5038c`, completed `scripts/check` through
self-analysis. It stopped at `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`, and
`LOCK-030`; this is a failed canonical gate, not release approval.
Standalone `scripts/package-check` passed.

The first full frozen trait-inventory run passed 78 cases: 28 accepted and 50
rejected. Its 8,058,874-byte payload has SHA-256
`0fc96eec571d70fb21f5c340addd9233877480433245d8e21425ed37df5782b1`.
The second run was stopped and produced no report. Repeatability is unverified;
`KD-TRANSPORT-IMPLS` and `KD-TRANSPORT-DETECTOR-IMPLS` remain unverified.
[The checkpoint index](evidence/transport-impls-checkpoint.json) preserves the
first report and completed/interrupted logs without claiming a second result.

The stopped checkout and any partial fixture copy remain under
`/mnt/volume_atl1_1787847588295/zrail-rc9/qualification-transport-impls` and
`/mnt/volume_atl1_1787847588295/zrail-rc9/evidence`. Preserve these on resumption;
use a fresh evidence directory for a new pair of runs. Do not reset snapshots,
reuse a partially mutated fixture, accept lock authority, or relabel this report
as qualification of the later declaration-inventory implementation.

After an explicit resumption, finish the trait validator and its adversarial
tests, reproduce both frozen runs at one pinned revision, and bind the evidence
before changing verification status. Complete declaration policy/evidence and
the remaining inventory, consumer bundles and release gates afterward.
