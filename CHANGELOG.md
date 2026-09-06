# Changelog

All notable zrail changes are recorded here for reviewed release notes.

## [Unreleased]

### Added

- Opt-in wiring-only facades, with separate unrestricted-import and
  public/crate/super re-export modes, plus reasoned exact-path mode selection.
- Test-facade structure independent of compilation reachability, test placement,
  test budgets, and execution identity.
- Facade coverage in schema 6 and effective modes in path explanations.
- Scoped package/path/role budgets with deterministic override rejection,
  independent target/soft/hard thresholds, warning-only targets, and test-facade
  limits independent of execution identity.
- Explicit bounded hard exceptions with required accountability forms and stale
  debt detection; optional authored baselines extend existing measured ratchets.
- Complete effective budget/debt reporting and protected comparisons for scoped
  limits, warning downgrades, baseline removal, and exception authority.
- Closed repository-file count/set, name, raw-text, and byte-equality assertions,
  with independent physical selection, bounded contained reads, complete coverage,
  path explanations, lock input binding, and protected semantic comparisons.
- Explicit UTF-8 validity for byte-equality policies, preserving legacy text-read
  preconditions while retaining binary equality by default.
- Exact raw-line presence and prohibition, with per-line normalization, complete
  occurrence totals, bounded samples, and protected comparisons against whole-file
  matching. Existing raw-text modes retain their meaning.
- Bounded authored TOML/JSON field predicates for presence, absence, nonempty
  strings, and exact typed values or ordered string arrays, with strict duplicate
  rejection, auditable observations, input binding, and protected semantic diffs.
- Immediate exact/allowed document key sets with explicit legacy non-table
  projection, required subject presence, complete counts, bounded key samples,
  and protected comparisons of accepted identities and type permissions.
- Typed immediate-field string prohibitions that preserve explicit legacy
  get/as_str projections while retaining required parent presence and input binding.
- Whole-Cargo.lock package counts and exact version/source/checksum inventories
  over the existing complete resolver, including unreachable nodes, persistent
  zero bans, bounded identity samples, coverage/explanation, lock binding, and
  protected comparisons that distinguish exact quantities from upper limits.
- Closed raw first-marker order, marker-bounded literal presence, and prefixed
  line-value allowlists, with complete quantities, bounded observations, input
  binding, and protected comparisons; these make no workflow execution claim.
- Exact authored Rust method-call inventories with selected-scope bounds and
  complete per-file/identifier count maps, independent physical selection,
  deduplicated locations, explicit cfg-world semantics, input binding, coverage,
  explanations, and protected quantity comparisons.
- Explicit written expression-path suffix inventories, including direct callees,
  function-value acquisition and path patterns, with distinct syntax claims and
  unchanged rc8 invocation authority. Qualification reuses the frozen collector.
- Exact file/subject owner sets independent of occurrence quantities, with
  required presence, persistent prohibitions, full count evidence, and protected
  comparisons against bounds and exact counts.
- Written path-segment membership across all authored Rust path contexts,
  reusing parsed path facts with exact physical owner sets and separate counts.
- Explicit authored import-rename inventories selected by source identifier,
  retaining alias destinations, same-name and underscore forms, exact sets,
  independent quantities, complete input binding, and protected review.

### Changed

- Analyzer/lock semantics advance to epoch 7 (lock schema 3); migration retains
  every previously supported epoch and adds the rc8 epoch 6 path.
- Strict facade relaxations and removed test-facade requirements are protected
  semantic grants. The existing declarative mode retains its rc8 meaning.
- Repeated immutable lexical-scope boundary queries reuse completed work within
  each source instance. Exhausted queries remain retryable and every analysis
  pass clears its cache; the self-hosted work ceiling is unchanged.

The rc9 replacement audit remains blocked; these capabilities do not establish
full downstream parity or authorize removal of any consumer guardrail.

## [0.0.3-rc.8] - 2026-09-04

### Changed

- Workspace producer and exact internal pins advance to `0.0.3-rc.8`; lock
  semantics remain epoch 6 (schema 3).
- The reviewed rc.7 source is repackaged under a new immutable candidate version
  without engine, policy, dependency, or workflow changes.

## [0.0.3-rc.7] - 2026-09-03

### Added

- Logical source mounts preserve package, crate, lexical imports, dependency
  authority, configuration predicates, origin, and mount-specific identity while
  retaining physical paths in diagnostics.
- Read-only lock-base discovery searches Git history for every revision matching
  the stored contract digest and reports whether local contract edits caused drift.

### Changed

- Workspace producer and exact internal pins advance to `0.0.3-rc.7`; lock
  semantics are epoch 6 (schema 3).
- Macro glob resolution now derives candidates from explicit exported macro sets,
  follows nested re-exports, and records genuinely unknowable sets explicitly.
- Rust invocation resolution queries the macro namespace independently of modules,
  types, functions, constants, and values with the same spelling.
- `zrail explain --path` now requires an existing path; deliberate future-path
  classification uses the explicit `--hypothetical-path` option.

### Fixed

- Included fragments inherit the include site's import and dependency-authority
  environment, and repeated mounts are resolved as distinct logical occurrences.
- Empty repository glob chains no longer invent macro candidates or interfere
  with compiler-provided macro authority.
- Lock migration mismatches now report both contract digests, actionable base
  recovery commands, candidate revisions, and local-edit causality.
- Leading-`::` macro paths bypass lexical imports, and public workspace
  proc-macro re-exports retain the re-exporting package's repository authority.
- Integration-test targets resolve their own library crate root, including bare
  macro imports from a named proc-macro re-export.
- Conservative macro resolution now provides the documented per-occurrence
  name-only fallback even when the same allowance carries exact source authority.
- Macro binding diagnostics name the canonical `resolution` setting.
- Crate source staging no longer requires Python 3.11's `tomllib` module.
- External macro globs resolve exact exports from checksum-matched local
  Cargo registry archives; missing, corrupt, conditional, globbed, or opaque
  export surfaces remain unresolved and require conservative review.

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
