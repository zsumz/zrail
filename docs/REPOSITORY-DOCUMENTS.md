# Authored document assertions

Unreleased rc9 supports a closed `document` predicate inside
[`repository.files`](REPOSITORY-FILES.md). It checks authored TOML or JSON fields.
It does not resolve Cargo inheritance, evaluate expressions, interpret commands,
or prove that a workflow or script executed. Existing Cargo analysis remains
responsible for package identity, resolution, and dependency topology.

```toml
[[repository.files]]
name = "inherited-version"
include = ["crates/public-package/Cargo.toml"]
reason = "This manifest must explicitly inherit the workspace version."
predicate = { kind = "document", format = "toml", path = ["package", "version", "workspace"], assertion = { op = "equals", value = true } }

[[repository.files]]
name = "publication-registries"
include = ["crates/public-package/Cargo.toml"]
reason = "The complete authored publication list has one approved registry."
predicate = { kind = "document", format = "toml", path = ["package", "publish"], assertion = { op = "equals", value = ["crates-io"] } }

[[repository.files]]
name = "description"
include = ["crates/public-package/Cargo.toml"]
reason = "Publication descriptions must contain text."
predicate = { kind = "document", format = "toml", path = ["package", "description"], assertion = { op = "nonempty-string" } }
```

The parser format is explicit. `path` is a sequence of literal, case-sensitive
table/object keys. An empty path selects the document root; a dot inside one
key remains a dot, not another traversal step. There are no wildcards, array
indices, filters, recursive descent, or executable expressions in this initial
selector. A missing intermediate key means the subject is absent. An intermediate
value of the wrong type fails the assertion, including `absent` assertions.

| Assertion | Meaning |
| --- | --- |
| `present` | The selected key exists. JSON null is present. |
| `absent` | The selected key does not exist. An empty physical file selection remains a valid prohibition. |
| `nonempty-string` | A string with at least one character after Rust's Unicode-aware `str::trim`. |
| `field-not-string` | The selected subject exists; its immediate literal `field` is absent or has a non-string type. |
| `equals` | Exact string, boolean, signed 64-bit integer, or complete ordered string array. Types, array order, duplicates, and cardinality matter. |
| `keys-exact` | Immediate object/table keys equal the complete `keys` set. Both missing and unexpected keys fail. |
| `keys-allowed` | Every immediate object/table key belongs to `keys`. Unused allowed keys are valid. |

Every assertion other than `absent` requires at least one selected file. Every
selected file must parse completely and satisfy the field predicate. String
equality does not trim or normalize whitespace; integer expectations reject
floating-point values such as `1.0`. Objects, mixed arrays, null, dates, floats,
and larger unsigned numbers are not supported equality expectations. They can
still be observed as present values. Unknown formats, operations, and fields
fail strict contract parsing.

Key sets compare literal, case-sensitive names independently of policy order.
They inspect the selected table only; nested or unrelated keys cannot satisfy
the inventory. Both modes require the selected subject to exist, even for an
empty expected set. By default a present non-table value fails. The explicit
`empty_on_non_table = true` option preserves legacy evaluators that project a
present value of another type to an empty key set. It never accepts a missing
subject. For example:

```toml
[[repository.files]]
name = "exact-dependencies"
include = ["Cargo.toml"]
reason = "Preserve the reviewed authored dependency names."
predicate = { kind = "document", format = "toml", path = ["dependencies"], assertion = { op = "keys-exact", keys = ["driver-core", "transport"] } }
```

`field-not-string` preserves the inventoried `get(field).and_then(as_str)`
projection. An object/table with no such field passes, as does an existing
field of a non-string type. A present non-object subject has no field and also
passes. The selected subject itself must exist, and wrong intermediate path
types still fail. Empty strings are strings and fail this predicate. This is
distinct from requiring field absence or declaring a valid Cargo dependency.
Its literal field name permits at most 1,024 bytes. Coverage exposes the
observed child type in `field_type`, independently of the parent selection.
Removing this constraint is a grant; redirecting its field stays protected.

TOML uses the existing TOML 1.0 parser with duplicate-key rejection and its
parser recursion guard. JSON additionally uses a bounded visitor that rejects
duplicate object keys, including differently escaped spellings of the same
key. Comments, trailing values, and nonstandard JSON syntax are rejected. Both
formats permit at most 100,000 value nodes and 64 nested levels, including
unselected portions of the document. Existing file input and content-work
limits apply. Policy paths permit at most 32 keys of 1,024 bytes each; expected
values and key sets permit at most 1,024 strings and 16 KiB of string bytes in
total. Duplicate expected keys are rejected.

The canonical policy identity remains `repository:file:<name>`. Coverage and
explain identify the claim as `authored-document`, expose the complete policy,
selected type, observed value when representable within 16 KiB, omission flag,
and intermediate selection errors. Omitted values never change evaluation.
Complete inspected input bytes remain hash-bound even when a field still passes.
Empty selections and absent keys remain visible.
Key observations include complete actual and unexpected counts, all missing
required keys, and lexical samples of at most sixteen actual/unexpected keys
and 16 KiB of JSON-encoded key bytes each. Oversized names are omitted from
samples, with explicit omission counts; they still participate in comparison.

`REP-FILE-007` identifies a structural predicate violation. Invalid UTF-8,
malformed documents, duplicate keys, and exhausted analysis bounds fail closed
with `REP-FILE-006`; they cannot produce a trusted partial lock or coverage report.
Protected diffs classify removed guards and weakened requirements as grants.
Changed exact values or array order can both grant and revoke authority;
changing literal key paths or parser formats remains protected as unknown.
Key-set diffs compare accepted sets: widening an allowlist grants authority,
exchanging exact identities can both grant and revoke, and policy ordering is
neutral. Allowing non-table empty projections is a grant when it admits new
inputs. An empty projection cannot satisfy a nonempty exact key set.

This initial subset supports the inventoried publication metadata fields.
YAML, selected array/object inventories, value sets, cross-field
relations, and selected workflow ordering remain explicit rc9 implementation
and qualification work. They must not be approximated using raw substring
searches or described as proven by these field checks.
