# rc9 guardrail replacement qualification

Release verdict: **blocked**. Inventory review and replacement qualification are
incomplete. No consumer guard has been deleted and no release artifact has been
published. The user approved retiring Rafter's private-name policy; see
[policy decisions](DECISIONS.md).

This branch implements strict facades, scoped size budgets, and bounded physical
file/raw-text predicates, UTF-8 equality, and bounded authored TOML/JSON field
assertions, whole-Cargo.lock package count/identity inventories, and exact authored
Rust method-call quantities. Verified rc9 replacement coverage remains incomplete.
See [exact reviewed assertion blockers](BLOCKERS.md),
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
contains **575 reviewed assertion instances**. The
[full tracked-file census](../../crates/zrail-testkit/tests/fixtures/rc9/census.json.gz)
and [summary](../../crates/zrail-testkit/tests/fixtures/rc9/census-summary.json)
record 9,792 files and 79,949 syntax candidates. Every tracked path is included,
including nested workspaces, sources outside `src/`, helper code, and malformed
detector fixtures. Non-Rust file contents, helper expansion, registries, and
opaque syntax still require review; this is not a completed assertion inventory.

| Repository | Tracked files | Rust files | Reviewed assertion instances | Implemented predicates | Verified detector assertions |
| --- | ---: | ---: | ---: | ---: | ---: |
| kafka-driver | 826 | 772 | 372 | 329 | 324 |
| Kafkars | 6,808 | 6,713 | 181 | 173 | 164 |
| Rafter | 2,158 | 1,897 | 22 | 0 | 0 |

The **488 verified assertions** comprise five facade predicates, four
kafka-driver budget assertions, 161 Kafkars size predicates/instances, and 44
kafka-driver raw/path assertions and detector fixtures, plus 33 publication
metadata assertions and parser preconditions, plus five authored dependency key
inventories, 35 authored dependency-field assertions and preconditions, and
36 authored provenance assertions and parser preconditions, and 20 whole-lock
cardinality/version/source/checksum assertions, plus 16 raw dependency/read
assertions, plus 125 qualification text assertions and read preconditions, and four transport
method/parser assertions.
Size instances include
all 141 measured baselines and three hard allowances. Their disposition
is **new engine capability**; verified counts for the other four dispositions
remain zero. Each registry instance binds its original TOML entry and the
independently failing helper predicate. The full size report preserves the
individual source digest, selected budget, baseline, and mutation diagnostics.
`KD-FACADE-LIVE` and `KF-FACADE-LIVE` remain unverified. No complete repository
has replacement qualification; passing a size family does not close other rails.

The uncompressed census SHA-256 is
`f90b74831614e4a660fdc8d1c008eb9f036fd81350f931478322af1378cab161`.
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
- Strict modes also cover newly added test `lib.rs`/`mod.rs` facades without
  requiring an existing exact-path declaration. The older modes retain their
  test-source selection behavior.
- Explain output exposes the effective mode and reason. Coverage schema 6 lists
  selected facade policies and every written violation without truncating totals.
- Mode weakening and removed or redirected facade authority remain protected.
  Analyzer semantics advance from 6 to 7; lock schema remains 3. Existing
  migration paths are preserved, including the newly supported rc8 path.

The frozen predicates are extracted byte-for-byte into the trusted Rust test
suite. Fifty-six differential comparisons and eight integration acceptance
tests pass. The imported Kafkars negative fixture is also byte-identical to its
source. Two further acceptance tests cover future test-facade discovery and
unchanged selection for existing contracts.

Strict same-revision rc8 reanalysis preserves all 36 compared authority entries.
The reviewed root lock has not been replaced or accepted. The final self-check
has four lock/input-drift diagnostics; the ordinary diff reports two protected
epoch-comparison unknowns. See the archived migration preview and qualification
report before proposing any authority update.

## Bounded physical file predicates

`repository.files` now supports required/forbidden counts, exact path sets,
explicit name-component/stem restrictions, deliberately raw UTF-8 predicates,
and byte equality. Selection is independent of source exclusions and compilation
worlds; unresolved directory/link boundaries and oversized/malformed inputs fail
closed. Coverage and explain expose the effective policy, quantities and inputs;
lock binding covers inspected bytes and physical selection. Protected comparison
respects quantifier direction and exact identities. See
[the file-policy contract](../REPOSITORY-FILES.md) and
[the qualified implementation evidence](evidence/files-index.json).

The [Kafka-driver file fragment](policies/kafka-driver.files.fragment.toml) now
has differential evidence for all 39 named predicates: 33 authored capability
tokens, four directory roots, a leading raw module marker, and absolute
component-stem restrictions. It compares 3,468 frozen path observations across
772 Rust files and 194 physical adversarial fixtures (66 accepts, 128 rejects).
The ledger also records five previously unexpanded detector assertions; all 44
associated assertion IDs link to the exact policy and expected diagnostics.
The byte-exact legacy helpers and registry entries are independently verified.

The [file parity index](evidence/file-parity-index.json) binds implementation
`ba7a52337ec435285e23a8ee1d64a095e90ab8e2`, source/policy/compiler/fixture identities,
and repeated byte-identical evidence. Its payload SHA-256 is
`edf2b1d64d99176a7cf53695fe0abfd8c9e3d186a146ffdf2a490e9a4cfaec63`.
Comments, strings, written aliases, qualification whitespace, every banned name,
directory stems, missing roots, and wrong entry kinds exercise native predicates.
The positive module marker requires a nonempty selection as the mission requires;
this is explicitly stronger than the old aggregate loop's vacuous empty result.

This is a complete demonstration of those physical/raw predicate surfaces, not
full Cargo/Rust source, semantic identity, lock, or execution qualification.
Structured TOML/JSON/YAML predicates, execution evidence, and the remaining
bounded Rust predicates remain release blockers.

## Scoped budgets and frozen per-instance evidence

Optional `source.rust.budgets` supplies package/path/role overrides, independent
hard limits, advisory soft limits, and warning-only targets. Exactly one scoped
override may match a physical file; same-tier competition fails checks, coverage,
explain, and lock construction. Exact-file hard exceptions require an explicit
bound and configurable owner/issue/tracking metadata. Authored ratchet baselines
remain exact even after a lock update; stale baselines and exceptions fail.
Contracts omitting this mode preserve rc8's existing ratchet interpretation.

The [kafka-driver fragment](policies/kafka-driver.sizes.fragment.toml) preserves
facade-first limits of 100/320/240. The
[Kafkars fragment](policies/kafkars.sizes.fragment.toml) preserves actual-package
test precedence and facade/implementation/test/auxiliary targets of 80/240/300/300,
soft limits of 120/360/500/500, and hard limits of 180/500/700/700. The three
Kafkars hard bounds are already implied by their existing exact baselines.
No size normalization or new legacy allowance was approved or applied.

Byte-exact frozen traversal and selector helpers agree with native inventory on
all **772 kafka-driver and 6,280 Kafkars governed Rust files**. The trusted
[size-only report](evidence/size-parity.json.gz) records **21,300 comparisons**,
including every valid file, relevant target/hard excess, growth, shrinkage,
stale baseline, and missing hard allowance. Its implementation revision is
`cb198bada58ef8f08bdc60bc7ac5cea0b2993a0a`, with an empty tracked diff. Generic
stock-check fixtures separately cover 54 numeric cases and four metadata
rejections. The unmodified frozen Kafkars and Rafter size test targets each pass
all five tests. Those legacy executions are retained qualification evidence.

These are partial policy fragments. The report deliberately makes no Cargo,
Rust-resolution, compilation-completeness, trusted-lock, or behavioral execution
claim. Remaining size-helper predicates and separately governed Rafter budgets
are explicit blockers; the complete downstream contracts still need qualification.

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
complete. [BLOCKERS.md](BLOCKERS.md) lists all **134 reviewed, unverified assertion
IDs and their causes**. The remaining candidate IDs are explicitly enumerated in
the census and need review before their disposition or complete assertion
expansion can be known. No unsupported generic assertion has been declared
behavioral merely to remove it from the release criterion.

## Metadata and retired-source expansion

The release metadata and release graph guards add **53 reviewed assertion
instances**: 32 metadata requirements, two metadata helper preconditions,
17 retired-source predicates, and two Rust parser preconditions. Their source
coordinates and expanded package/module instances are bound in the ledger.
Thirty-three metadata requirements now have bound frozen differential evidence;
the parent-path helper and retired-source assertions remain unverified. This
review exposed the license UTF-8 precondition
and the retired-tree scanner's ignored read errors and unbounded directory-link
traversal. Those limitations cannot silently become exclusions.

The metadata translation contains 35 native file/document policies, including
three explicit parser/presence preconditions over the selected manifests. The
original assertion body and read/parse helpers execute unchanged in trusted
qualification, with only the workspace root supplied by the fixture harness.
Both repeated reports at `db11ebe40f3ffe969939785b34f2acf880c6b79e` are byte-identical:
`9895d7213b58d96e144fdc362c6c9fc0d86274f8b93ebdb47b1bfe9423340485`. Every one of the 11 frozen input files is hash-bound;
35 positive and 99 negative physical fixtures prove intended diagnostics.
The [metadata evidence index](evidence/metadata-index.json) records the same-code
canonical gate and archive results. No complete downstream contract or execution
claim is inferred from this authored-metadata qualification.

## Dependency guard expansion

All 38 assertion/failure candidates in Kafka-driver's `dependency.rs` are now
expanded into 55 reviewed instances, including six previously reviewed bans.
The 49 additional rows remain unverified. They preserve exact versus subset
ordinary-dependency keys, explicit feature order and types, registry/private
publication distinctions, parser and array preconditions, raw CI occurrence
counts, and exact trimmed Git attributes lines. This is source/configuration
inspection even where the function name mentions protocols or simulation.

`KD-DEP-SIM-NO-VERSION-STRING` accepts absent or non-string versions in the
legacy detector; a simple absent-key check would change that policy. The key-set
helper maps a present non-table dependency value to an empty set; a missing
entry panics at the preceding Value index. This distinction was verified against
the pinned TOML implementation, not inferred from map_or_else alone. These
projection semantics remain explicit gaps, along with strict trimmed-line
presence and whole-lock line-name extraction. Neither generic Cargo topology
nor a permissive substring predicate is claimed to subsume them.

Registry bindings now include exact scalar and whole-array values as well as
indexed entries. The offline verifier checks 193 instantiated bindings against
the frozen TOML bytes; `index = null` in the JSON ledger selects the whole value.

## Verified authored dependency key inventories

`KD-DEP-DRIVER-KEYS`, `KD-DEP-CORE-KEYS`, `KD-DEP-TRANSPORT-KEYS`,
`KD-DEP-SIM-KEYS`, and `KD-DEP-PROBE-KEYS` now link to five native policies in
[`kafka-driver.key-sets.fragment.toml`](policies/kafka-driver.key-sets.fragment.toml).
Four require exact authored names; the deterministic core preserves its subset
rule. Present non-table values project to empty sets exactly as in the frozen
helper, while missing subjects still fail. These policies do not infer Cargo
identity or absorb dev/build/target dependencies.

Two frozen runs at `c86f5173c90e564b204420789faf2d16ef17807a` produced identical
reports: five baseline observations over six immutable inputs, plus 130 fixture
outcomes (22 accepted, 108 rejected). All 25 original keys were separately
deleted and aliased; malformed documents, unrelated tables, wrong types,
missing inputs, case changes, and undisplayed unexpected names exercise the
intended policies. The [bound evidence](evidence/key-sets-parity-index.json)
records exact source, policy, fixture, code, compiler, and input identities.
The complete dependency file still has other unverified assertions.

## Verified authored dependency fields

Thirty native policies in
[`kafka-driver.dependency-fields.fragment.toml`](policies/kafka-driver.dependency-fields.fragment.toml)
cover 35 additional reviewed assertion IDs: exact declaration versions,
workspace inheritance, private/public publication settings, optional/default
feature flags, ordered feature arrays, parser preconditions for all five
manifests, and the simulator's exact non-string version projection. Public
array type/count/registry assertions map to their equivalent conjunction;
feature array preconditions map to the same complete typed equality already
required by the original guard. Each linkage records that proof explicitly.

Two runs at `c7ff4588f029169c10cce191be3bac3ceaec9e2d` produced identical
[bound evidence](evidence/dependency-fields-index.json): 30 frozen observations,
six immutable inputs, and 151 fixture outcomes (34 accepted, 117 rejected).
Six complete original assertion bodies and their read/parse helpers execute
in trusted tests only. Reordering, omission, duplicate feature entries, wrong
versions/types, missing parents/files, and malformed inputs all exercise their
intended native policies. No source patch or authority grant was applied.
Whole-lock/provenance, CI/line checks, source guards, and complete downstream
qualification remain separate open work.

## Protocol provenance inventory expansion

The frozen `protocol_provenance.rs` now has 43 further reviewed instances,
covering all 30 census candidates when helper-call links are included. These
include five exact workspace version/type pairs, five exact-version prefixes,
five ordinary inheritance table/type/count conjunctions, the optional TLS
conjunction, root/probe `patch`/`replace`/`target` prohibitions, and lock/parser
preconditions. The previously reviewed 20 lock identity assertions now bind
their exact original version/checksum registry entries and helper invocations.

The review also identified an implicit predicate in the lock filter: every
array entry must have a name key, even when unrelated to the selected package.
A present non-string name is ignored by the old filter. Native Cargo parsing
rejects that malformed lock shape more strictly; this distinction is recorded
as `KD-PROVENANCE-LOCK-PACKAGE-NAMES` and requires explicit qualification.
Whole-lock checks cannot be replaced solely by reachable dependency policies.
Of these 43 new rows, 36 authored document assertions now have verified
differential evidence. Five registry-prefix and two whole-lock preconditions
remain unverified. Complete repository inventory remains open.

## Authored protocol provenance parity

At `51901c18681e15b1f14db2b61a2db057c678a90d`, the 27 generated
provenance document predicates accepted the untouched frozen source. Two
trusted runs produced byte-identical reports for 161 fixtures: 39 accepted and
122 rejected through the intended document diagnostic. This verifies 36
assertion IDs, including exact workspace-reference type/count/field
conjunctions and the root/probe bans on any `patch`, `replace`, or `target` key.
The parser policies retain required file presence alongside absent-key rules.

See [bound provenance evidence](evidence/provenance-index.json). The five
generated whole-lock policies now have their own 20-assertion frozen
differential qualification, recorded below. No package-node claim is inferred from successful document parsing.

## Whole-lock protocol identity parity

At `5251b4f056741768eb667c74d6e0949257ca843b`, all five whole-lock policies accepted
the frozen kafka-driver graph. Fifty differential fixtures produced 20 accepts
and 30 intended `DEP-LOCK-001` rejections, with no old/new disagreement. This
verifies the 20 `KD-LOCK-<package>-{COUNT,VERSION,SOURCE,CHECKSUM}` assertion IDs.
Deletion, duplicated names, and same-count identity substitutions remain
resolvable graphs, so unrelated Cargo parsing failures cannot satisfy parity.

The [evidence index](evidence/lock-parity-index.json) binds both identical runs,
the five native workspace identities, twelve unchanged source inputs, extracted
legacy bodies, policy, toolchain, and test binary. The complete lock has 83
nodes; selection does not depend on reachability. Extra unrelated valid nodes
and qualified incoming edges remain legal. The six lexical `KD-LOCK-BAN-*`
assertions and seven registry-prefix/lock-shape preconditions remain separate.
No source-analysis certificate or downstream lock was produced by this slice.

## Raw dependency read preconditions

Review expanded the shared `read` helper into `KD-DEP-READ-LOCK`,
`KD-DEP-READ-CI`, and `KD-DEP-READ-ATTRIBUTES`. These require physical readable
UTF-8 inputs independently of absent-marker rules. The three new rows and
thirteen raw predicates now have verified frozen differential evidence. The raw lock bans retain line spelling and deliberately
do not claim parsed Cargo identity. CI literal checks do not prove execution.

## Raw dependency differential parity

At `58e5835c82915144ebd1f147e536a9cbcb0af170`, all 16 policies
accepted the frozen inputs. Two byte-identical runs cover 157 fixtures:
85 accepted and 72 rejected through `REP-FILE-004` or `REP-FILE-005` for the
intended policy. See the [bound evidence](evidence/raw-dependency-index.json).

The original raw lock extractor and complete CI/attributes bodies execute
unchanged. Full-line matching preserves exact names, Unicode trimming, CRLF,
comments, and suffix behavior. Raw CI checks still accept markers inside
comments or unrelated fields, as the legacy assertions do. They establish no
workflow execution or YAML claim. Required file reads are qualified separately
from absence bans. The two original name-extractor detector assertions remain
open pending their own complete replacement-fixture linkage.

## Qualification guard expansion

Review of all assertions in frozen `tests/guardrails/qualification.rs` adds
102 assertion instances from 84 previously unreviewed assertion/failure sites,
plus 23 newly instantiated UTF-8 read preconditions across 24 inputs. The CI
read reuses `KD-DEP-READ-CI`. `KD-CI-ENFORCEMENT` remains its existing separate
policy decision and cutover blocker. All 125 new rows now have bound differential
evidence; no execution claim is inferred from source test names or vocabulary.

The expansion identified these requirements, now implemented by closed native
raw-text predicates:

- `KD-QUAL-095` and `KD-QUAL-096`: strict order of the first raw fetch/gate/archive
  markers, with required marker presence.
- `KD-QUAL-097`: the offline marker must occur inside the first gate-to-package
  byte interval, including its start and excluding its end. Whole-file presence
  does not preserve this assertion.
- `KD-QUAL-228-01` through `KD-QUAL-228-06`: every raw trimmed `image: ` line must
  contain one of the two exact pinned values. Zero selected lines is legal to
  this predicate; each selected file still has an independent read requirement.

The other 93 assertions use the existing whole-file literal modes. The
[125-policy fragment](policies/kafka-driver.qualification.fragment.toml) and
[qualification evidence](evidence/qualification-parity-index.json) bind every
assertion, exact input, original test body, and intended diagnostic. Two clean
runs at `7c6c01d0af8bdeee598e87c3c8fae97ce72b9a0c` produced identical reports:
923 fixtures, 410 accepted and 513 rejected. The audit verifier rejects changed
selectors or predicate values, incomplete/duplicate fixture sets, unrelated
input drift, wrong diagnostics, and overstated execution claims. These checks inspect CI,
package metadata, scripts, compose files, and smoke sources; they do not execute
Cargo, npm, smoque, a shell, or Kafka. The runners and behavioral scenarios remain
in the downstream cutover plan. Raw conditions that accept comments must not be
presented as parsed YAML or successful execution evidence.

## Transport inventory assertion expansion

All ten assertion sites and both parser-failure sites in frozen
`tests/guardrails/transport_authority.rs` are reviewed. The five live boundaries
remain `KD-TRANSPORT-*`; five new `KD-TRANSPORT-DETECTOR-*` IDs bind the exact
adversarial inventory expectations, and `KD-TRANSPORT-PARSE-PRODUCTION` /
`KD-TRANSPORT-PARSE-FIXTURE` preserve strict item-file parsing. At this review checkpoint all twelve were unverified; the four method/parser
assertions are now qualified below. Eight transport assertions remain open.

The associated inventory counts every selected `ExprPath`, including function
references, by physical file and final two identifier segments. It selects
thirteen suffixes and requires the exact eight-entry count map. Method calls
use four written names and the exact eleven-entry count map without resolving
receiver types. The implementing-type/trait/file inventory is an exact
three-member set; duplicate identical tuples are collapsed by the legacy guard.
Neither a name-only owner rule nor three arbitrary impls preserves that set.

Owner-file presence visits every original `syn::Path` context, including
attribute, visibility, macro-name, generic-argument, and qualified-self paths.
An expression-only inventory would miss these cases. Explicit renames compare
the source identifier, including same-name and underscore renames; plain and
glob imports remain outside that predicate. The original traversal includes all
authored cfg branches and does not parse opaque token bodies. These syntax
claims must remain distinct from resolved identity, expanded source, and
compilation-world quantities. Existing call-owner invocation semantics remain
unchanged. Strict item-file parser preconditions cannot be replaced silently by
zrail's separately supported expression-fragment parsing.

Authored method quantities and strict file parsing are now implemented for
`KD-TRANSPORT-METHODS`, `KD-TRANSPORT-DETECTOR-METHODS`, and the two transport parse
preconditions. The [native inventory contract](../RUST-INVENTORIES.md) preserves
authored cfg branches and physical occurrence identity independently of semantic
receiver resolution. Twenty source-bound synthetic collector comparisons and
ten integration tests pass. The [method policy fragment](policies/kafka-driver.transport-methods.fragment.toml)
preserves eleven exact map entries and twenty occurrences. Full frozen selection
and original detector-fixture parity were still open at that implementation
checkpoint; the completed evidence is recorded below. At that native-method checkpoint, the ledger recorded 498 implemented and 484
verified assertions out of 559 reviewed instances.


## Remaining driver helper and placement assertions

Nine additional assertion sites now have stable IDs: `KD-FACADE-PARSE`,
`KD-REGISTRY-PARSE`, `KD-REGISTRY-SCHEMA`, the three `KD-TRAVERSAL-*` I/O
preconditions, and the three `KD-TEST-PLACEMENT-*` assertions. This brings the
ledger to 568 reviewed instances, 500 implemented predicates, and 484 verified
assertions. All nine new assertions remain verification blockers. Existing strict
facade and authored-inventory parsing cover two implementation requirements;
they are reused rather than replaced with another parser.

The frozen test-placement guard examines top-level items only. Its inline-module
condition looks for the case-sensitive literal `test` in the rendered `cfg`
token list. It therefore selects `cfg(not(test))` and `cfg(feature = "latest")`
as well as `cfg(test)`. Nested items, qualified test attributes, and opaque macro
bodies have different original behavior. The detector's exact ordered two-item
result is a separate assertion. Semantic test reachability alone does not
preserve these source predicates, and no normalization has been approved.

The shared helper's required typed TOML fields, supported schema, and every
fallible directory/entry/type observation are recorded separately. Its remaining
invocation expansion and the wider non-Rust review stay open; accounting for the
visible failure/assertion sites does not complete the repository inventory.


## Raw-process collector review expansion

Six additional `RF-PROCESS-*` records identify the strict parser precondition,
two normalization expectations, and three crate-alias/macro/import detector
assertions. At that review checkpoint the ledger had 574 reviewed assertions, 500 implemented, and
484 verified. Rafter's five existing live predicates now describe the exact
collector and bind its helper source hashes. The eighteen required tuples remain
unchanged, and all eleven raw-process assertions remain blockers.

Rafter's `Expression` label includes ordinary `syn::Path` visits, not only
`ExprPath` or direct calls. Imports flatten groups, globs and source-side renames;
rename targets and visibility paths do not participate. Normalization uses the
exact declared module owner and strips raw-identifier prefixes, while a separate
predicate compares written and normalized segments. The macro ban flattens
identifiers across nested token groups, ignores punctuation and literals, and
looks for adjacent `execution`, `process` identifiers. Those semantics differ
from both expansion provenance and an authored method-call inventory.

`RC9-INVENTORY-FALLIBLE-CALLS` records a discovery gap: the existing census does
not identify `.expect`/`.unwrap` preconditions such as parsing the raw detector
fixture at `process_scenarios.rs:406`. Extend the trusted census before claiming
complete assertion coverage. Remaining process-format, lifecycle, reaper,
launcher-selection and shared module-graph assertions also require review; this
source file does not receive a whole-file disposition.


## Fallible-call discovery checkpoint

The expanded census at `e01891c5843757f9f9cb5e1f5f39cfc21ca6e583` records 79,949 candidates,
including 7,794 newly identified `expect`, `expect_err`, `unwrap`, and
`unwrap_err` calls (4 kafka-driver, 189 Kafkars, 7,601 Rafter). Two fresh offline
runs produce identical 37,598,886-byte payloads. Every previous candidate and
reviewed source identity is preserved, as are all frozen file hashes and Git
identities. [The checkpoint](evidence/census-fallible-index.json) binds code,
compiler, test binary, prior artifact and both logs. Three focused tests cover
chained calls, qualified calls, raw identifiers, function-value non-calls,
comments, literals, opaque boundaries and default trait helper context.

`RC9-INVENTORY-FALLIBLE-CALLS` is closed as a discovery defect.
`RF-PROCESS-DETECTOR-PARSE` now identifies the newly discoverable fixture parse
precondition. At that census checkpoint the ledger had 575 reviewed, 500 implemented and 484 verified
assertions; it remains incomplete. New candidates still require individual
review, including behavioral preconditions and custom methods with matching
names. Included fragments and opaque macro bodies remain explicit review work.


## Verified frozen transport method and parser parity

At `151270ffa2285800833f2295abcfc61aa689d9ca`, two complete frozen-scope
method runs produced byte-identical reports. All 772 original Rust inputs are
bound by hash; the original traversal and native selectors agree on 479
non-test physical files. The exact eleven-entry map contains twenty occurrences.
All 76 fixtures agree: seven accepted and 69 rejected, including duplicate,
missing, relocated and substituted occurrences, all six roots, test exclusions,
qualified and nested syntax, opaque macro boundaries, and both strict parser
failures. The original detector fixture and parser execute in trusted tests.

`KD-TRANSPORT-METHODS`, `KD-TRANSPORT-DETECTOR-METHODS`,
`KD-TRANSPORT-PARSE-FIXTURE`, and `KD-TRANSPORT-PARSE-PRODUCTION` now bind this
[report and qualification index](evidence/transport-methods-index.json). Their
intended diagnostics are `RUST-INVENTORY-001` for quantities and
`RUST-INVENTORY-002` for incomplete file parsing. The payload SHA-256 is
`431fb863939196180696673b66350ab81538867595e35f8b1d6d846e55d15bf7`.
The evidence validator checks every original input and complete per-fixture map;
twenty tampering cases cannot substitute unrelated failures or partial counts.

At that evidence checkpoint the ledger had **575 reviewed / 500 implemented / 488 verified**. These are
authored syntax assertions, with no receiver identity, Cargo compilation, lock,
execution, or complete repository claim. The other eight transport assertions
remain blockers, and the downstream file cannot yet be removed.


## Written expression-path inventory implementation

The stock inventory family now adds `written-expression-paths`, selecting exact
final two written identifiers. It reuses located pre-projection path/call facts
and authored AST membership; the existing method facts, resolver, and direct-call
ownership semantics remain unchanged. Both call callees and function-value
acquisition participate. The original collector also visits path patterns,
which are represented by `syn::ExprPath`; the ledger now states that context
explicitly. Comments, strings and macro-token payloads remain outside this claim.

Twenty-two source-bound differential syntax cases and three further integration
tests pass, including aliases, qualified-self syntax, generic arguments, cfg
branches, nested items, path patterns, same-count substitutions, repeated mounts,
source-byte binding, coverage and explain. Eight core inventory schema/diff tests
pass. The [partial expression-path policy](policies/kafka-driver.transport-expression-paths.fragment.toml)
preserves thirteen suffixes and eight exact per-file quantities.
`KD-TRANSPORT-ASSOCIATED` and `KD-TRANSPORT-DETECTOR-ASSOCIATED` are implemented
but remain unverified pending complete frozen-source and exact detector parity.
The ledger is **575 reviewed / 502 implemented / 488 verified**.


## Nonstandard Rust fragment discovery gap

`RC9-INVENTORY-FRAGMENTS` identifies seven already-hashed `.inc` inputs that the
`.rs`-only census did not parse: one Kafkars imported-macro bypass fixture and six
Rafter artifact/Maelstrom verifier fragments. The
[explicit input registry](../../crates/zrail-testkit/tests/fixtures/rc9/additional-rust-inputs.json)
binds their frozen repository, revision, exact path and bytes, plus the reviewed
including/fixture source identities. This supplies discovery input only; it does
not attest macro authority or classify independent verification as a generic
architecture policy.

The trusted census now supports these reviewed item fragments without guessing
from arbitrary extensions. Five census tests pass, including stale source/review
identities, duplicate registrations, unsupported syntax and path selectors. The
committed census still needs regeneration before the discovery gap can close;
new candidates will require assertion-level review.
