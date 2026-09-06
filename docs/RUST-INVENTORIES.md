# Rust syntax inventories

`source.rust.inventories` adds required quantities to zrail's existing parsed
source facts. It is optional and empty in older contracts. Every rule requires
a reason, file selectors, a world, a typed subject, and an assertion. The
canonical identity is `rust:inventory:<name>`.

```toml
[[source.rust.inventories]]
name = "transport-methods"
reason = "The reviewed backend owns exactly these written wake sites."
include = ["src/**/*.rs"]
exclude = ["src/**/*_test.rs"]
world = "authored"
subject = { kind = "written-methods", names = ["wake_handle", "poll_io"] }
assertion = { kind = "exact-counts", counts = [
  { path = "src/backend.rs", name = "wake_handle", count = 2 },
] }
```

The complete observed map must equal the declared map: two `wake_handle`
method-call expressions in that file and zero selected methods everywhere else
in the selection. Moving a call, changing its method name, adding one, or
deleting one violates the rule. Entries must have unique `(path, name)` pairs
and positive counts. An empty `counts` array is an active prohibition.

For a selected-scope quantity use `assertion = { kind = "count", minimum = 1 }`
or `assertion = { kind = "count", minimum = 1, maximum = 3 }`. Bounds are
inclusive. `minimum` defaults to zero, and `maximum` is optional. A zero minimum
without a maximum is rejected as vacuous. `maximum = 0` remains valid when its
subject or selected files are absent. A positive requirement fails when its
subject disappears, including when every selected file disappears. An exact
single-file selector gives a per-file quantity without a separate language.

## Claim boundary

The initial subject is `written-methods`, and the only supported world is
explicit `authored`. Counts are exact for authored method-call syntax:

- Each physical `(path, written method, identifier span)` counts once, even
  when source is mounted repeatedly. Overlapping selectors do not duplicate it.
- All authored cfg branches participate, including `cfg(test)`, `cfg(any())`,
  mutually exclusive branches, and branches excluded by feature selection.
  This is a physical authored inventory, not a union of compilation-world counts.
- Nested functions, closures, blocks, test items, const-generic expressions,
  and parsed attribute name-value expressions participate. Path selectors
  determine which physical files are governed; structural role and test
  reachability remain unchanged.
- The identifier is case-sensitive and retains an authored `r#` prefix.
  Current policy identifiers accept bounded ASCII Rust identifier spellings.
  Generic method arguments do not change the written method identifier.
- `value.poll()` counts; `Type::poll(value)` and `let f = Type::poll` do not.
  Receiver identity is not resolved or implied. Existing call-owner semantics
  remain unchanged.
- Comments, strings, attributes' token payloads, macro definitions' token
  bodies, and opaque macro invocation tokens do not supply method-call
  expressions. No expansion or runtime claim is made. Existing macro authority
  and completeness rules continue to apply independently.

Selections reuse the bounded physical repository scanner independently of
`repository.exclude`, Cargo source filtering, and mount reachability. Every
selected entry must be a physical `.rs` file with complete Rust *file* facts.
Selected excluded files, foreign workspace sources, links, unreadable files,
invalid syntax, and expression-only include fragments fail closed. A fragment
cannot silently substitute for a failed file parse. Separate contracts govern
separate workspaces; inventory selectors do not authorize crossing them.

## Audit, limits, and review

`RUST-INVENTORY-001` reports a quantity or exact-map violation under the named
policy. `RUST-INVENTORY-002` reports incomplete inventory analysis; no partial
lock or coverage report can be produced. JSON coverage's `rust_inventories`
records contain full policy, explicit claim and quality, every selected input
digest, every file/subject count, complete occurrence totals, sixteen sampled
locations, and an explicit omitted-location count. Explain reports the matching
policy and complete scope quantities. Display sampling never affects decisions.

The parser reuses located method facts and indexes membership in the original
AST. It supplies missing syntax observations in contexts outside semantic
traversal and excludes macro-token observations. Existing macro-aware facts
remain unchanged, and this additional index is absent in contracts without
inventories.

The analysis certificate binds all inspected source bytes, even comments and
files with no matches, together with the observations and policy. Authoring no
inventory preserves the previous certificate behavior. This is part of the
unreleased rc9 semantics epoch 7 and coverage schema 6.

Limits are 1,024 rules, 64 selectors per rule, 128 subjects per rule, 4,096 exact
pairs per rule, and 50,000 declared occurrences per rule. Across one analysis,
physical selection allows 8,000,000 queries and 250,000 selected entries;
inventory observation allows 64 Mi fact comparisons, 256 MiB unique source
bytes, and 50,000 matched occurrences including repeated policy observations.
Input digests are reused. Existing parser, source, contract, and traversal limits
also apply. Exceeding a bound is incomplete analysis, never a truncated pass.

Protected review recognizes removed guards, loosened bounds, replacing exact
maps with totals, and narrower prohibited scopes as grants. Changing an exact
count in either direction changes accepted states and records both grant and
revocation. Expanding positive-presence scope weakens its requirement; expanding
an exact map's scope adds zero-count requirements for newly selected pairs.
Unproved selector changes remain protected unknowns. Declaration ordering is
irrelevant; reason changes remain protected unknowns.

Other relationships, semantic identities, and compilation-world inventories
remain explicit rc9 replacement blockers until implemented and qualified.
