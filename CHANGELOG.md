# Changelog

All notable zrail changes are recorded here for reviewed release notes.

## [0.0.3] - 2026-08-24

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
