# Changelog

All notable zrail changes are recorded here for reviewed release notes.

## [Unreleased]

## [0.0.3-rc.6] - 2026-08-31

### Added

- Repository-source macro authority, including cross-crate exports, bounded
  provider/helper input digests, reviewed extra input patterns, and an explicit
  no-ambient-input attestation. This is reviewed authority, not a runtime sandbox.
- Digest-bound cross-revision lock migration for source fixes that make the new
  engine unable to analyze the old base. Contract grants remain separately reviewed.
- Distinct host-side proc-macro and proc-macro-test compilation domains, excluded
  from production-only runtime profiles but governed by all-reachability profiles.
- Deterministic self-hosted projection-work qualification and advisory timing smoke.

### Changed

- Workspace producer and exact internal pins advance to `0.0.3-rc.6`; lock
  semantics are epoch 5 (schema 3).
- Cargo feature-world exactness is limited to the verified conservative subset;
  unsupported split host/target contexts fail closed and have Cargo oracle fixtures.
- Declarative facade conventions allow central data declarations and required
  proc-macro entrypoints while continuing to reject implementation bodies elsewhere.

### Fixed

- Migration enforces Git-tree gitlinks under submodule-deny policy; allowed
  links remain opaque and content-bound instead of becoming empty directories.
- Migration reports normalize written and canonical temporary paths so Windows
  and aliased paths retain stable reviewed report identities.
- Repository macro closures with external dependencies require exact validated
  Cargo lock resolution. Targeted capture prunes reserved output directories
  before descent without letting source exclusions hide implementation inputs.
- Leafness follows logical source occurrences across parent and sibling includes,
  requires a clean namespace, and stays separate for repeated mounts. Exact
  field-type const paths resolve in the value namespace.
- Exact type shape now filters guarded fields and child modules per compilation
  domain, rejects possible shape, checks every governed world, and reports domain
  identities. Active item-replacing attributes cannot claim exact representation
  through namespace-only authority. Coverage uses the same resolved shapes.
- Repository macro input digests bind owned non-Rust files even when source
  exclusions hide them; explicit input authority rejects missing/escaping inputs,
  symlinks, and undeclared ambient assumptions. Qualified names cannot bypass review.
- Macro resolution is occurrence-specific; target-predicate module mounts,
  include-mounted fragments, and macro/module namespace collisions no longer
  downgrade unrelated exact builtin origins.
- Clone/Copy spelling-equivalent policies compare neutrally; negative impls do
  not count as duplication grants, and redundant prohibitions are rejected.
- Fresh worktrees tolerate absent disposable cache tags, documentation lock drift
  uses lock diagnostics, and test-only `use super::*` remains a supported convention.

## [0.0.3-rc.5] - 2026-08-29

### Added

- Exact workspace-wide Cargo feature worlds, runtime-neutral async-syntax
  policy, facade-aware glob-import policy, exact Rust type/authority shape and
  non-duplication rails, and policy-declared field-mutation ownership.
- Non-executing bulk test-mirror planning, strict plan-bound result ingestion,
  deterministic receipt rendering, and bulk receipt verification.

### Changed

- Lock state now uses schema 3 and semantics epoch 4, with direct reviewed
  migration from every released prior epoch.
- Contract formatting and schema migration preserve authored comments, blank
  lines, key order, spacing, and generated-section markers.

### Fixed

- Ordinary Rust binding and typed-place resolution now retain prelude,
  re-export, conditional-compilation, typed-variable, and nested-field
  identities without silently accepting unresolved authority.
- Included impl fragments now bind associated definitions to the exact current
  trait across nested modules and equivalent aliases without leaking identity
  across different traits.

## [0.0.3-rc.4] - 2026-08-25

### Fixed

- Crate publication preflight now models unpublished workspace dependencies as
  checksum-bound crates.io sources, preserving canonical registry lockfiles and
  exact archive bytes before any upload.
- The mismatched `zrail-rust` RC.3 archive was yanked before any downloads; RC.4
  is a fresh candidate with registry-equivalent package evidence.

## [0.0.3-rc.3] - 2026-08-24

### Fixed

- Crate publication preflight now resolves unpublished workspace dependencies
  from the reviewed checkout while proving the exact publish-mode archive bytes
  before any registry probe or upload.

## [0.0.3-rc.2] - 2026-08-24

### Fixed

- Clean-container release qualification now creates an atomic onboarding
  baseline before checking, preserving fail-closed missing-lock enforcement for
  ordinary contract-only initialization.

## [0.0.3-rc.1] - 2026-08-24

### Added

- Input-scaled base and derived source contexts, sparse include projection,
  typed incompleteness diagnostics, reviewed analysis budgets, workload
  metrics, memoized binding queries, a lock-bound completeness certificate,
  and a deterministic 10,001-physical-file qualification regression.
- Exclusion-aware, contract-first initialization; atomic optional baselines;
  exact entrypoint role overrides; and selector-specific hygiene ratchets.
- Offline `Cargo.lock` identities and graph traversal for exact macro-source
  authority, shortest-path transitive dependency prohibitions, and checksum-
  and revision-bound ambiguity rejection.
- Schema-2 exact contract fragments, deterministic `zrail fmt`, explicit
  `zrail migrate-config`, exact item-macro namespace manifests, and scoped
  adjacent-epoch `zrail migrate-lock` reports. Exact item-macro lock state binds
  the definition content, syntax guard, and every Cargo compilation domain.
- Generic type-construction, written-method, field read/write/mutable-borrow,
  and field-authority ownership from one shared source-operation model.
- Exact production-to-test mirrors, strict versioned execution receipts, and
  deterministic schema-2 `zrail coverage` audit output with an enabled-rail
  census, spans, guards, compilation domains, analysis quality, dependency
  paths, and exclusions.

### Changed

- Lock state now uses schema 2 and semantics epoch 3. Migration acceptance is
  digest-bound and remains separate from grant acceptance.
- Macro policy emits `resolution` and `namespace_effect`; schema 1 remains
  readable for explicit migration.
- Diagnostic retention is named `--max-findings`; `--limit` remains a
  deprecated compatibility alias.
- Protected tags now package, verify, checksum, attest, publish, and re-download
  all three crates.io artifacts before making the GitHub release visible.

### Security

- Incomplete analysis, ambiguous resolved identities, stale evidence, changed
  receipt bytes, and cross-epoch authority cannot produce or silently replace
  trusted lock state.

## [0.0.2] - 2026-08-23

### Added

- `zrail baseline` for adopting existing contracts without accepting new debt.
- Independent module-documentation and hygiene ratchets with tightening debt
  baselines.
- Production-reachability filtering for compile effects and capability owners.
- Macro glob resolution, conservative binding, normalized policy names, and
  one structured binding diagnostic per invocation.
- Cargo-compilation-domain and lexical macro authority across normal, unit-test,
  integration-test, benchmark, example, and build-script targets.
- Exact file-role overrides and scoped authority for item-position macros.
- Exact aggregate diagnostic status, configurable `--limit` retention,
  per-rule totals, and report wire schema 2.
- Workspace-boundary discovery that governs observed extras while isolating
  excluded and nested workspaces.
- Checksummed prebuilt archives, attestations, and build provenance for seven
  supported CLI targets.

### Changed

- Resolved architecture locks now use semantics epoch 2. Existing epoch-1
  locks must be regenerated with the current zrail before check or protected
  review. Comparisons spanning epochs report unknown lock authority, never a
  grant or debt change.

### Fixed

- Workspace discovery, module ownership, cfg propagation, macro binding, and
  protected lock mutation now fail closed across their adversarial edge cases.
- Domain-dependent textual macro shadowing now requires authority for every
  feasible origin and content-binds reviewed repository implementations.
- Literal and verified generated includes now preserve occurrence-specific
  textual macro order and fail closed on unresolved cross-file aliases.

## [0.0.1] - 2026-08-22

### Added

- Deterministic architecture contracts, exact locks, semantic authority diffs,
  protected proposal review, and path-specific explanations.
- Rust and Cargo dependency-layer, source-boundary, generated-source, facade,
  test-placement, size, and hygiene checks.

### Security

- Unknown configuration, unresolved source relationships, stale policy, and
  unreviewed grants fail closed.
