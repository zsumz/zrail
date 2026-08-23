# Changelog

All notable zrail changes are recorded here for reviewed release notes.

## [0.0.2] - 2026-08-23

### Added

- Exact aggregate diagnostic status, bounded rendering, per-rule totals, and
  report wire schema 2.
- Reachability-aware Cargo, module-edge, ownership, effect, and macro authority
  across production and test domains.

### Changed

- Resolved architecture locks now use semantics epoch 2. Existing epoch-1
  locks must be regenerated with the current zrail before check or protected
  review. Comparisons spanning epochs report unknown lock authority, never a
  grant or debt change.

### Fixed

- Workspace discovery, module ownership, cfg propagation, macro binding, and
  protected lock mutation now fail closed across their adversarial edge cases.

## [0.0.1] - 2026-08-22

### Added

- Deterministic architecture contracts, exact locks, semantic authority diffs,
  protected proposal review, and path-specific explanations.
- Incremental-adoption baselines with tightening size, module-documentation,
  hygiene, and test-placement ratchets.
- Exact Cargo workspace, dependency, source-graph, macro-origin, reachability,
  ownership, file-role, and generated-source analysis.
- Configurable diagnostic retention with exact aggregate status and per-rule
  totals.
- Checksummed prebuilt archives and build provenance for seven supported CLI
  targets.

### Security

- Unknown configuration, unresolved source relationships, stale policy, and
  unreviewed grants fail closed.
