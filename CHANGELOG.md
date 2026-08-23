# Changelog

All notable zrail changes are recorded here for reviewed release notes.

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

### Changed

- Resolved architecture locks now use semantics epoch 2. Existing epoch-1
  locks must be regenerated with the current zrail before check or protected
  review. Comparisons spanning epochs report unknown lock authority, never a
  grant or debt change.

### Security

- Unknown configuration, unresolved source relationships, stale policy, and
  unreviewed grants fail closed.
