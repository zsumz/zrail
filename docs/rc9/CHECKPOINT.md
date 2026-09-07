# Active rc9 goal checkpoint

The 118-commit history is now PGP-signed in the user's commit style. See the
[signed-history correspondence](HISTORY.md) before resolving the historical
commit identifiers below. The recorded evidence and release blockers remain
unchanged by that metadata-only rewrite.

The user resumed work and explicitly set a completion goal on 2026-09-07.
See [the completion order](COMPLETION.md). Release verdict remains **blocked**.
The current ledger is **672 reviewed / 558 implemented / 512 verified**;
160 reviewed assertion instances and the three discovery blockers remain open.
No complete consumer bundle or full downstream qualification exists yet.

The two trait-set assertions now have two fresh byte-identical 78-case reports
at the original pinned implementation `5393665`. Their evidence index is
`evidence/transport-impls-index.json`; the stopped report remains intact.
The evidence validator passes 13 methods covering 116 adversarial mutations.

The mixed kafka-driver route-test review accounts for 31 previously unlisted
instances. Six stock file policies now verify its five source assertions and
eleven compile-time inputs. At `359fd5621f42e106615a66bb7a665a85cff3ff41`, two
byte-identical 189-case reports accept 43 and reject 146 cases each. Each run also
compiles the complete original source-only test once and proves 22 intended
missing/invalid-UTF-8 compilation failures. The artifact validator passes two
methods covering 42 tamper cases. The fifteen behavioral route assertions remain
unverified. See `evidence/route-source-typed-index.json`.

Rafter source-boundary review adds 66 instances: 41 static boundary/input
requirements and 25 contracts linked to the approved private-name retirement.
The new bounded raw line-prefix set preserves the original matcher in 992
positive/negative comparisons. The 158-rule partial Rafter fragment accepts all
194 frozen source inputs using 57.52 MiB of the unchanged content-work bound.
The retirement is not a verified native replacement, and these rows remain
unverified pending final evidence/cutover linkage. Twenty-one audited non-Rust
spans bind frozen shell/document bytes separately from parsed Rust identities.

The final working code passed **1,609 Rust tests**, formatting, Clippy, rustdoc,
structure and complete self-analysis: 1,051 files, 1,646 base contexts,
1,194,894 work, zero unresolved. Seventeen Rust tests are explicitly ignored in
the ordinary suite; the frozen Rafter boundary test ran separately and passed.
`scripts/check` rejects exactly `LOCK-008`, `LOCK-016`, `LOCK-026`, `LOCK-028`
and `LOCK-030`. Standalone `scripts/package-check` and both Git whitespace
checks passed. The initial gate caught a diagnostic-expectation error in the
new UTF-8 fixture; its corrected full rerun has zero test failures.
[The evidence index](evidence/rafter-boundary-index.json) binds the exact pending
inputs and every log. No input changed during execution. This does not qualify
an uncommitted tree as a release revision.

The unchanged Rafter source-boundary baseline passes two tests, and the public
API/docs baseline passes eight. The latter guard and `scripts/reference-source-check`
were read completely; their [API](inventory/RAFTER-PUBLIC-API-DISCOVERY.json) and
[reference](inventory/RAFTER-REFERENCE-DISCOVERY.json) findings remain explicit
assertion-expansion blockers. API warnings are enforced errors with 41 reviewed
site-pattern allowances. Reference hard allowances lack measured caps in the
legacy script; no bounded replacement allowance has been accepted automatically.

The user explicitly approved skipping signing for local commits. The six
prepared slices are saved as `6f5b3e9`, `5b2ca58`, `1f18bf5`, `1a3d5da`,
`bc2c8de`, and `7fb1093`; per-command signing overrides leave the global
configuration intact. That exact committed tree was independently requalified
in `/root/zrail-rc9-qualification-7fb1093`: 1,609 Rust tests passed, 17 ignored,
formatting/Clippy/rustdoc and complete self-analysis passed, and standalone
archives passed. The same five protected lock diagnostics still stop the
canonical gate. `evidence/committed-check-index.json` binds this clean committed
run; the earlier pending-tree evidence remains historically distinct.

Route qualification and artifact validation are saved in `8c198a0`, `f552b53`
and `0e1fce7`. The failed first route run remains preserved. Source, policy and
compiler-input identities are explicit; no artifact claims full downstream
qualification. Local signing no longer blocks progress.

The idle 7 GiB `/root/zrail/target` cache was preserved at
`/mnt/volume_atl1_1787847588295/zrail-rc9/root-zrail-target-preserved-20260907`,
with its original path retained as a symlink. No cache contents were discarded.

Continue with the retained route execution receipt, retired declaration-policy
translation, and the remaining kafka-driver inventory and full policy bundle. Continue the remaining consumer inventories
before treating the engine capability list as complete. No downstream guard was
removed, no authority accepted, and no publication, push or release tag occurred.

## Previous stopped checkpoint

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
