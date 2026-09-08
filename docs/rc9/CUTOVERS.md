# Downstream cutover plans

Status: **blocked** for every repository. No downstream source, contract, lock,
workflow, or guard has been changed. No complete replacement-policy bundle exists
yet. The facade, size, file, metadata, and key-set fragments in `policies/` are review material for
individual capabilities;
they must not be installed as a repository's complete contract.

`cutovers.json` in the rc9 testkit binds candidate checker paths and reviewed
assertion links to the frozen revisions. Every candidate remains conditional:
an unreviewed assertion in a mixed file prevents removing that file. There are
no checkers or helper dependencies certified removable now.

## kafka-driver first

| Candidate evaluator | Replacement and prerequisites |
| --- | --- |
| `tests/guardrails/facade.rs` | `rust:facades` with `wiring-only`; exact test-facade declarations for `tests/guardrails.rs`, `tests/guardrails/mod.rs`, and `tests/support/mod.rs`. Close `KD-FACADE-LIVE` and verify full discovery. |
| `tests/guardrails/file_size.rs` | Verified `KD-BUDGET-*`: 100 facade, 240 production, 320 test with facade-first selection; assemble with the remaining complete policy bundle. |
| `tests/guardrails/module_contract.rs` | Verified `repository:file:kd-module-leading-doc` and `repository:file:kd-module-names`, including all four original detector assertions. Preserve raw marker semantics and the bound absolute checkout prefix; complete repository qualification remains required. |
| `tests/guardrails/capability.rs` | Verified `repository:file:kd-capability-01-01` through the four frozen token groups and `repository:file:kd-capability-root-01..04`, with exact mapping in the ledger. Comments and strings retain their original participation; complete repository qualification remains required. |
| `tests/guardrails/transport_authority.rs` | Twelve assertions are verified by `rust:inventory:kd-transport-methods`, `rust:inventory:kd-transport-expression-paths`, `rust:inventory:kd-transport-owners`, `rust:inventory:kd-transport-renames`, and `rust:inventory:kd-transport-impls`. Qualify the assembled bundle before removal. Preserve function-value `ExprPath` counts, written renames, method quantities, and exact implementing-type/trait sets. |
| `tests/guardrails/dependency.rs`, `protocol_provenance.rs` | Preserve whole-lock bans and exact package/provenance counts (`KD-LOCK-*`), the five verified `repository:file:kd-dep-*-keys` inventories and 30 verified policies in `kafka-driver.dependency-fields.fragment.toml` for versions, feature order/flags, inheritance, publication, and parser inputs. CI/line predicates, protocol provenance, and remaining assertion expansion are still open. |
| `tests/guardrails/release_graph.rs` | Twelve declaration, raw-construction and required-input assertions retain the original 144-case evidence. Seven tree assertions now bind the corrected `kafka-driver.retired-boundary.fragment.toml`, two repeated ordinary/physical/injected-error reports, and explicit fail-closed differences. Use this corrected fragment when assembling the final bundle; the historical file-only fragment is insufficient. Keep the checker installed until complete discovery and full downstream qualification. |
| `tests/guardrails/release_metadata.rs` | `kafka-driver.metadata.fragment.toml` supplies 35 native policies with 33 assertion IDs verified, including authored inheritance, ordered publication lists, UTF-8 license equality, required files, and deliberate raw markers. Close the parent-path helper proof and full repository qualification before removal. |
| `tests/guardrails/qualification.rs` | `kafka-driver.qualification.fragment.toml` supplies 125 verified raw predicates/read preconditions; reuse `repository:file:kd-dep-read-ci` for CI readability. Close `KD-CI-ENFORCEMENT` with the enforced lane and qualify the complete repository bundle before deletion. Actual command execution remains separately evidenced. |
| `src/reactor/direct_plaintext/cluster_runtime/route_state_test.rs` | Six verified `repository:file:kd-route-*` policies replace only the source-only function and its compile inputs after full qualification. The fifteen runtime assertions now bind original execution and the proposed patched-snapshot native mirror. Review the one-line explicit-trait helper patch, then qualify the final downstream policy/CI/receipt revision. Retain the actual routing scenario and mixed file. |
| `tests/guardrails/test_location.rs` | Qualify sibling/test placement against the detector's actual cfg syntax. Record any stronger native reachability semantics explicitly. |
| `tests/guardrails/support.rs`, `mod.rs`, `tests/guardrails.rs` | Remove only after every dependent assertion and detector fixture has a verified replacement. |

The mission explicitly requires replacing `KD-CI-ENFORCEMENT`'s current blanket
`!workflow.contains("zrail")` predicate. The proposed downstream change must add
a required CI lane that verifies the protected rc9 archive/provenance, runs
`zrail check`, reviews the contract/lock diff against a trusted base with grants
denied, and verifies applicable execution receipts. Installing zrail is not the
lane. A lane invoking `tests/guardrails.rs` is side-by-side evidence during the
transition and cannot count as a stock replacement.

Retain workspace behavioral tests, protocol fixtures, the deterministic simulator,
real-broker probes, smoque scenarios, performance evidence evaluation, and the
actual runners `scripts/check`, `scripts/qualify-packages`,
`scripts/qualify-latest-compatible`, `scripts/prefetch-release-dependencies`, and
`scripts/verify-release-repository`. Mixed scripts need assertion-level extraction
before removing their static evaluator portions. Their execution cannot be
replaced by matching text in a workflow.

After all guard imports are gone, review whether `syn`, `serde`, and `toml`
remain used elsewhere before proposing dev-dependency removal. None is certified
removable by this audit slice.

## Kafkars

The candidate surface starts with
`crates/kafka-client-guardrails/tests/`, its shared `support` modules, detector
fixtures, `guardrails.toml`, and `contracts/invariants.toml`. It is not the whole
inventory: source-only assertions outside that crate remain discovery blockers.

The facade fragment uses `wiring-reexports`, preserving private import and
`pub(self)` rejection. Close `KF-FACADE-LIVE` across configured roots, including
four explicitly selected integration-test facades. Preserve the independently
classified production/test/auxiliary size roles; no shared 300-line normalization
is proposed.

`KF-FETCH-ORDER-*` and `KF-FETCH-PRESENCE-*` remain stock-engine blockers. Keep
their raw selection/order semantics visible while qualifying the requested
function-scoped code predicates. Comments or unrelated nested code cannot become
evidence for the proposed code predicates. Ownership allowlists do not cover
these assertions.

Translate ownership registries from canonical source identities, including their
field read/write/borrow distinctions and explicit mutation-method lists. Complete
the invariant planned/enforced, normative statement, evidence uniqueness, exact
test reference, and gate relationship review before migrating the registry.

Retain actual protocol, broker, state-machine, cancellation, bounded-resource,
and simulation tests and all their fixtures. Retain the execution portions of
`scripts/check`, `scripts/check-rust-lint`, and `scripts/check-rust-test`.
`scripts/check-architecture` may become the stock enforcement invocation only
after the ledger is complete; wrapping the guardrail crate would not replace it.
Remove the dedicated checker crate or its parser dependencies only after every
mixed assertion has been discharged and downstream qualification passes.

## Rafter and independent workspaces

Root, `reference`, `bench-compare`, and `fuzz` require explicit governance and
invocations. Preserve the root workspace's existing completeness boundaries;
do not grant cross-workspace inheritance to make one root command sufficient.

| Candidate surface | Prerequisites |
| --- | --- |
| `crates/rafter/tests/readability_architecture_guard.rs` and support | Close `RF-FACADE-LIVE`, `RF-TEST-FACADE-LIVE`, and `RF-WIRE-*`; review every other assertion in this mixed file. Data declarations remain legal under its existing facade predicate. |
| `crates/rafter/tests/invariant_tooling_architecture_guard/` and support | Close all `RF-PROCESS-*` requirements, including the exact 18 file/context/path entries; retain independent producer/verifier and intra-package domain boundaries. |
| `crates/rafter/tests/source_boundary.rs` | Close all 41 `RF-BOUNDARY-*` static/input requirements with the native raw boundary fragment and complete traversal evidence. Only the private-pattern branch and parser detector belong to the 25-instance approved private-name retirement. The shared traversal and static guards remain until native qualification completes. |
| Public API/docs, publication, CI test inventory, size, and invariant catalog guards | Complete assertion and registry-instance inventory, document predicates, exact public sets, receipt input/outcome binding, and source/test mounting evidence. |
| `scripts/reference-source-check` | Close `RC9-INVENTORY-RF-REFERENCE-SCRIPT`: replace handwritten publishable-patch closure, patch-path presence and source-size/input/allowance predicates. Preserve separate 400/700/1000 implementation and 700/1100/1500 auxiliary tiers; six uncapped legacy hard allowances need an explicit bounded migration decision. Keep patch argument construction and formatting, lint, test and rustdoc execution. |
| `scripts/reference-package-boundary-check`, `scripts/verify-action-pins`, related detector tests | Inventory every static manifest/workflow/path assertion; retain actual package construction and independent artifact verification. |
| `scripts/private-name-scan` | Its eighteen `RF-PRIVATE-SCAN-*` contracts retire under `RC9-DECISION-RF-PRIVATE-NAMES`, alongside three `RF-PRIVATE-RELEASE-*` document contracts and four source-boundary extension/parser instances. Remove the manual release invocation in the later downstream PR. Preserve independent package construction and artifact verification. |

Retain TLA+ model checking and telemetry, Maelstrom workloads and history checks,
Raft simulation/burn-in, codec/storage fixtures, reference process/package tests,
`scripts/cargo-test-exact`, and the independent verification programs under
`crates/rafter-invariants`. Their structural contracts are review work; their
behavior and evidence verification must continue to execute.

## Required cutover sequence

1. Close all assertion discovery, unsupported predicate, and policy-input blockers.
2. Review complete declarative bundles and any narrowly identified source changes.
3. Qualify untouched snapshots and any separately identified patched snapshots;
   run legacy and stock enforcement side by side with intended diagnostics.
4. Review and accept authority only through the existing protected process.
5. Open separately authorized downstream changes with exact deletion maps,
   enforced CI, retained execution tools, fixture lineage, and receipt producers.
6. Requalify each downstream revision before deleting any old evaluator.
