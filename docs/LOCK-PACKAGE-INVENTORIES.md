# Whole-lock package inventories

`[[dependencies.lock_package]]` checks every parsed node in the root
`Cargo.lock` with one literal, case-sensitive Cargo package name. Selection
includes nodes that no current manifest reaches, development/build-only nodes,
and nodes unused in the selected feature worlds. Edges do not multiply nodes.
This uses the existing offline Cargo resolver and never invokes Cargo.

```toml
[[dependencies.lock_package]]
name = "reviewed-wire"
package = "wire"
reason = "The reviewed wire release has one immutable registry identity."
assertion = { kind = "exact", identities = [
  { version = "1.2.3", source = "registry+https://github.com/rust-lang/crates.io-index", checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
] }

[[dependencies.lock_package]]
name = "retired-runtime"
package = "retired-runtime"
reason = "No node from this retired runtime may remain anywhere in Cargo.lock."
assertion = { kind = "count", count = 0 }
```

Replace the illustrative checksum with the reviewed package checksum. An exact
inventory requires the complete version/source/checksum set for the selected
name. Array order has no meaning; duplicate expected identities are invalid.
Multiple versions or sources remain distinct. Omitting `checksum` requires
checksum absence. Local workspace nodes use `path+<repository-relative-directory>`
(the root package uses `path+.`); registry and Git nodes retain Cargo's source
string, including a Git commit. These are locked identities, not semver ranges,
import spellings, or dependency aliases.

`count` requires exactly the authored quantity. Zero remains an active ban when
the package is absent. An empty exact inventory has the same prohibition.
Positive counts and nonempty exact inventories fail when their package
disappears. Count mode deliberately makes no version/source/checksum promise.

Both modes require a complete, valid Cargo.lock, even for zero bans. Missing
inputs, duplicate package identities, invalid provenance, ambiguous dependency
references, and missing active workspace nodes fail before candidate lock
creation. Existing Cargo parsing requirements remain unchanged. A legacy
raw-line scan is a separate text predicate: this family does not count text in
comments, unrelated TOML fields, or multiline strings as package nodes.

`DEP-LOCK-001` identifies a count or identity-set violation, with the canonical
policy ID `dependency:lock-package:<name>`. Coverage schema 6 includes
`lock_packages`; `zrail explain --path Cargo.lock` shows the same effective
policy and observations. JSON contains the complete selected count, input hash,
whole-lock node count, exact analysis quality, every missing expected identity,
and bounded observed/unexpected samples with explicit omitted totals. Human
output summarizes selectors, expectations, quantities, justification, and
result. The complete Cargo.lock bytes and contract sources use the existing
analysis certificate and lock binding; byte drift is `LOCK-039`.

The family permits at most 1024 rules, 1024 expected identities and 16384
expected identity bytes per exact rule, and counts through the resolver's
100000-node limit. Name/version/source literals are bounded to 256/1024/4096
bytes respectively (package names use existing Cargo-name validation).
Comparison work sums one unit per selected node plus its version/source/checksum
bytes across every rule, with a fixed 64 Mi-unit ceiling. Excess work fails
closed. Each observed or unexpected display sample is limited to 16 identities
and 16384 encoded JSON bytes; omitted nodes still participate in all decisions.

The optional family defaults to empty and is omitted when serialized empty,
preserving rc8 contract meaning. Removing a rule grants authority. Exchanging
exact identities, changing an exact count in either direction, or retargeting
a package grants the newly accepted states and revokes the former states.
Replacing a nonempty exact set with its quantity is a grant; the reverse is a
revocation. Reordering a set or exchanging the two zero-ban forms is neutral.
Changed justification remains unknown and requires review. Existing config
formatting and migration preserve authored comments and layout.
