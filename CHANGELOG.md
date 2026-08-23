# Changelog

All notable zrail changes are recorded here for reviewed release notes.

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

## [0.0.1] - 2026-08-22

### Added

- Deterministic architecture contracts, exact locks, semantic authority diffs,
  protected proposal review, and path-specific explanations.
- Rust and Cargo dependency-layer, source-boundary, generated-source, facade,
  test-placement, size, and hygiene checks.

### Security

- Unknown configuration, unresolved source relationships, stale policy, and
  unreviewed grants fail closed.
