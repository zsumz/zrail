# Typed registry conversion and native schema boundary

Status: implementation and focused fixtures only. `KD-REGISTRY-PARSE` and
`KD-REGISTRY-SCHEMA` remain unverified. No complete policy bundle is qualified.

The exact original `Guardrails`, `Paths`, `Budgets`, `Dependencies` and
`Capability` structures plus `load_guardrails` and `read` are extracted from
frozen `tests/guardrails/support.rs:10-54` and `:60-70`. The complete structures
matter: the earlier whole-lock harness's smaller registry type is not evidence
for this full-registry precondition. Every deserialized field is observed in
the new test projection. Snapshot verification binds both exact excerpts.

The source requires schema `1` as a `u32`. Budgets are `usize` on the supported
64-bit targets, but the original TOML parser first imposes signed 64-bit integer
bounds. Unknown fields are ignored only after valid TOML parsing. Required
strings may be empty; arrays may be empty or contain duplicates. Their later
live constraints remain separate assertions. Wrong element types, missing
fields, duplicate keys, invalid UTF-8 and unreadable files fail.

`scripts/rc9_registry.py` preserves these typing distinctions, ignores unknown
legacy fields and admits only unchanged interpreted values from the pinned
registry. Changed budgets, roots, names, array order, versions or checksums do
not become authority. Its explicit 4 MiB input limit is stronger than the
original loader; no unbounded error-path parity is claimed. The public CLI
creates only a fresh, labeled review file and refuses to overwrite an output.
It does not run a consumer checker, update a lock or install a policy.

## Native schema-test carrier

The carrier uses the existing good-contract scaffold and data from seven
unchanged fragments plus the corrected `traversal-inspection` fragment, checking
every interpreted registry field against their selections and values.
It contains 119 file policies, five exact lock
identities and three budget overrides. Source schema `1` maps to explicit
native schema `1`; native schema `2` also loads this same carrier, but source
schema `2` is still rejected. Unknown native contract keys are rejected, unlike
unknown source-registry keys. These are representation differences, not exact
acceptance-set parity.

The scaffold is a parser fixture, not the downstream repository layout or a
full consumer contract. No rule is excluded: `kd-source-traversal` now uses the
explicit `inspect` predicate and `entry = "any"`. The earlier traversal fragment
and its predicate-only evidence remain byte-identical historical inputs.

## Blocking discovery: RC9-NATIVE-TRAVERSAL-CONTRACT

The unmodified traversal fragment uses `entry = "any"` with
`predicate = { kind = "count", minimum = 0 }`. Loading a carrier containing it
through the public native contract loader fails with:

```text
file assertion "kd-source-traversal" has an empty or vacuous count constraint
```

The earlier traversal reports establish isolated predicate/physical behavior,
not the assembled fragment's acceptance by the contract loader. Their original
bytes and limited claims remain unchanged. The native regression substitutes
the unchanged historical rule for corrected inspection and still requires this
exact failure. Vacuous counts are not made valid by the new predicate.

The representation defect is fixed in source: complete inspection has a closed,
loadable policy, public report/lock tests and the same 133 physical boundary
fixtures run after public contract loading (19 require enforced Unix permissions).
The 96 injected scanner failures remain separate from observed OS behavior.
Narrowed or removed inspection is protected; no new authority is accepted.

Next: bind renewed repeated clean signed-producer traversal and registry reports,
including frozen source coverage and the now-complete schema-test carrier.
`RC9-NATIVE-TRAVERSAL-CONTRACT` remains open for that qualification renewal,
not because the corrected representation is still unloadable. Only afterward
close the two registry rows; no full consumer contract is qualified by this fix.

## Focused evidence

- 249 source cases execute the complete unchanged original loader and compare
  both failure stage and every normalized typed field with converter fixtures.
  Cases cover every required field, nested capability entry, schema/numeric
  boundary, ignored field, empty/mixed/duplicated/reordered array and empty
  version/checksum string.
- 27 native cases independently execute the public contract loader. Schemas
  1 and 2 load with identical policy after schema normalization; unsupported,
  malformed, missing and unknown fields fail. The known traversal-fragment
  rejection remains a separate explicit regression.
- Three registry Rust tests and seven Python tests exercise source binding, the two
  matrices, deterministic fixture regeneration, frozen values, the stronger
  input bound, and actual CLI output/overwrite refusal.

These are focused tests, not two repeated clean-producer reports or an accepted
replacement certificate. Failed extraction/compile attempts and the first
numeric/carrier mismatch remain diagnostic logs under
`target/rc9-completion-20260907/registry-focused*.log`; they are not relabeled
as qualification passes.

The first self-analysis also identified unreviewed `json!` usage in the new
test projection. It was replaced with ordinary typed serialization, without a
macro grant. The first full gate was deliberately stopped after this finding;
`registry-final-check.log` is an interrupted run, not a completed qualification.
