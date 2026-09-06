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

Use `exact-owners` when a reviewed assertion requires distinct ownership
locations without fixing the number of occurrences at each location:

```toml
assertion = { kind = "exact-owners", owners = [
  { path = "src/backend.rs", name = "wake_handle" },
] }
```

Every listed `(path, name)` pair must occur at least once, and every unlisted pair
must have zero occurrences. Repeated occurrences at a listed pair preserve its
membership; deleting its last occurrence or moving it to another file fails.
An empty `owners` array remains an active prohibition. Entries must be unique,
selected exact Rust paths and subjects; quantity fields are rejected. The same
4,096-pair limit applies. Coverage retains every actual count, input digest and
sampled span independently of the ownership-set decision.

## Claim boundary

The only supported world is explicit `authored`. The `written-methods` subject
counts exact authored method-call syntax:

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

The `written-expression-paths` subject counts authored `syn::ExprPath` nodes
whose final two written identifier segments match a configured suffix:

```toml
subject = { kind = "written-expression-paths", suffixes = ["ConnectionSet::new", "Source::register"] }
assertion = { kind = "exact-counts", counts = [
  { path = "src/owner.rs", name = "ConnectionSet::new", count = 1 },
] }
```

Each suffix has exactly two bounded ASCII identifiers. Leading qualification and
generic arguments do not affect the suffix; raw-identifier prefixes and case
remain significant. `ConnectionSet::new()`, `let f = ConnectionSet::new`, and
`crate::transport::ConnectionSet::<T>::new` participate. `Alias::new` does not
match `ConnectionSet::new`, even if the alias resolves to that type.
`<T as Source>::register` matches `Source::register`; `<Source>::register`
has a one-segment path and does not. Qualified-self types are visited separately,
so nested const-generic expression paths still participate.

This is a syntax-node inventory: Rust path patterns use the same `syn::ExprPath`
representation and participate, including `match x { ConnectionSet::new => ... }`.
Ordinary type paths, imports, and macro-token payloads are outside this subject.
All authored cfg branches and parsed expression contexts participate as above.
Each physical `(path, suffix, complete path span)` counts once. The claim is
`authored-rust-expression-path-syntax`, never invocation or resolved identity.
Changing a direct call into a function-value acquisition preserves this quantity;
rc8 direct-call owner restrictions retain their independent meaning.

The `written-paths-containing` subject matches every authored `syn::Path`
containing a configured exact identifier segment:

```toml
subject = { kind = "written-paths-containing", names = ["ConnectionSet"] }
assertion = { kind = "exact-owners", owners = [
  { path = "src/set_owner.rs", name = "ConnectionSet" },
] }
```

Type, expression, trait, attribute-name, restricted-visibility, macro-name,
pattern, generic-argument and qualified-self paths participate. Import `UseTree`
leaves and opaque token contents are separate syntax and do not participate.
A declaration named `ConnectionSet` is not itself a path. An unqualified pattern
parsed as a binding is not a path; a qualified path pattern is. Attribute
name-value expressions participate, while token payloads do not. All authored
cfg branches participate, and case and raw prefixes remain significant.

Each physical `(file, selected name, complete path span)` counts once. Repeating
the same name inside one path does not duplicate that pair. A path containing
two different selected names contributes one match to each subject; the total
is the number of path/subject matches. Nested paths in generic arguments or
qualified-self types are separate nodes. Owner sets discard only quantity when
deciding membership; coverage retains all quantities. The explicit claim is
`authored-rust-path-membership-syntax`, independent of resolved identity.

The `written-import-renames` subject selects explicit `UseTree::Rename` nodes
by their original written identifier. It preserves the alias destination as
part of each observed identity:

```toml
subject = { kind = "written-import-renames", names = ["Source"] }
assertion = { kind = "exact-owners", owners = [] }
```

This prohibits `use path::Source as Alias`, `Source as Source`, and `Source as _`,
including grouped/nested imports and every visibility. It does not select plain
or glob imports, `Other as Source`, `Source::{self as Alias}`, extern-crate
renames, type aliases, comments, strings, or opaque macro tokens. Raw prefixes
and case remain significant. All authored cfg branches and parsed contexts,
including nested code, const-generic blocks, and attribute expressions, participate.

Exact counts and owners use the closed identity spelling `Source as Alias` in
their `name` field. Both identifiers must be bounded supported spellings; the
alias may also be `_`. Changing the destination changes identity even when
the occurrence count stays constant. Counts retain duplicate renames at distinct
physical spans; exact owner sets collapse only identical file/source/alias
memberships. Each sampled span covers the complete `source as alias` leaf.
The claim is `authored-rust-import-rename-syntax`, independent of semantic alias
resolution. Existing binding facts retain their earlier meaning.

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

The parser reuses located method/path/call facts and indexes membership in the
original AST. It supplies missing syntax observations in contexts outside semantic
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
inventory observation allows 64 Mi fact and written-segment comparisons, 256 MiB unique source
bytes, and 50,000 matched occurrences including repeated policy observations.
Input digests are reused. Existing parser, source, contract, and traversal limits
also apply. Exceeding a bound is incomplete analysis, never a truncated pass.

Protected review recognizes removed guards, loosened bounds, replacing exact
counts with owner sets or totals, and narrower prohibited scopes as grants. Changing an exact
count in either direction changes accepted states and records both grant and
revocation. Expanding positive-presence scope weakens its requirement; expanding
an exact map's scope adds zero-count requirements for newly selected pairs.
Changing a required owner set records both grant and revocation; removing a
member removes its presence requirement and adds a prohibition. Exact empty
counts, exact empty owner sets, and a zero upper bound have the same meaning.
Changing subject kinds and unproved selector changes remain protected unknowns. Declaration ordering is
irrelevant; reason changes remain protected unknowns.

Other relationships, semantic identities, and compilation-world inventories
remain explicit rc9 replacement blockers until implemented and qualified.
