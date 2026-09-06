# Bounded raw-text structure

Unreleased rc9 adds three closed `repository.files` predicates for existing
source-only checks of scripts, workflows, and other UTF-8 files. All matching
is case-sensitive and deliberately includes comments and string contents.
These predicates do not parse Rust, YAML, or shell, or establish execution order.

```toml
[[repository.files]]
name = "fetch-before-gate"
include = [".github/workflows/qualification.yml"]
reason = "Preserve the reviewed first-marker ordering assertion."
predicate = { kind = "literal-order", before = "run: cargo fetch --locked", after = "run: scripts/check" }

[[repository.files]]
name = "offline-gate-interval"
include = [".github/workflows/qualification.yml"]
reason = "The offline marker belongs inside the reviewed gate interval."
predicate = { kind = "literal-between", start = "run: scripts/check", end = "run: scripts/qualify-packages", contains = 'CARGO_NET_OFFLINE: "true"' }

[[repository.files]]
name = "worker-images"
include = ["worker-config.txt"]
reason = "Only the complete reviewed raw suffix values are permitted."
predicate = { kind = "line-values-allowed", prefix = "image: ", values = ["reviewed-image-a", "reviewed-image-b"], normalization = "trim" }
```

`literal-order` requires both markers and compares the byte offsets of their
first occurrences with strict `<`. Later occurrences cannot repair an earlier
violation. `literal-between` requires the first start offset to precede the first
end offset and searches only that interval, including the start offset and
excluding the end offset. Matching text outside the interval cannot satisfy it.
Both predicates require at least one selected regular file. Missing markers,
equal/reversed bounds, and missing selected files fail with `REP-FILE-008`.
The first offsets remain explicit in coverage even when a marker is absent.

`line-values-allowed` uses Rust `str::lines`, preserving its LF/CRLF and trailing
newline semantics. Each line is normalized with `none` (default), `trim-start`,
or `trim`; then a literal prefix selects it and is removed. The entire remaining
suffix must equal one allowed value. Neither values nor prefixes are implicitly
case-folded, unquoted, or interpreted as document syntax. An empty value set
prohibits all selected lines. Zero selected lines or files remains valid for
this prohibition; use a separate path/read predicate when presence is required.
Duplicate allowed values, CR/LF in prefixes or values, and unsupported
normalization modes fail contract validation.

Markers and prefixes are nonempty and at most 16 KiB each. Allowed value sets
contain at most 1,024 distinct values and 16 KiB of combined value bytes. Existing
file, aggregate input, selection, and content-work limits also apply. Invalid
UTF-8 or an exhausted bound yields incomplete analysis (`REP-FILE-006`) and
cannot produce a trusted partial lock or coverage report.

Coverage schema 6 and explain output include the complete policy, selectors,
input hashes, precise raw claim, and `text_structure` observations. Interval
matching reports the full count and at most sixteen original byte offsets.
Line-value matching reports complete selected/unauthorized counts, at most
sixteen unauthorized values within a 16 KiB encoded sample, and the omitted
count. Oversized values still participate in comparison and totals. Every
inspected byte is bound, including content outside a selected interval.

Removing a guard, replacing bounded presence/order with an entailed whole-file
presence check, shortening an interval's required literal, expanding an allowed
value set, or narrowing a prohibited-line file scope is a protected grant.
Reordering the same allowed value set is neutral. Arbitrary marker, prefix,
normalization, or interval-boundary changes retain protected uncertainty when
accepted-state inclusion is unproven. Existing contract meanings and defaults
remain unchanged; this is part of unreleased analyzer semantics 7.
