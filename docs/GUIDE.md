# zrail guide and trust model

This guide covers zrail's policy, lock, review, and qualification model. See the
[README](../README.md) for the short path from installation to a first check.

## Model

zrail keeps three concerns separate:

```text
zrail.toml   human-authored architecture
zrail.lock   resolved, content-bound architecture state
Cargo.lock   Cargo's selected versions, checksums, and Git commits
```

`zrail check` reads these files and repository source without modifying them.
Cargo remains the package resolver; zrail does not duplicate that job.

The architecture lock records the exact state needed for checks and semantic
diffs, including:

- normalized direct dependency declarations and exact reviewed `Cargo.lock`
  identities;
- the complete analyzed inventory, exclusions, contract fragments, and
  analyzer semantics, including Cargo feature definitions and exact feature
  worlds;
- reviewed gate bytes and their declared behavioral inputs;
- exact test-mirror receipt bytes, reviewed inputs, and execution identity;
- generated-source provenance;
- content-bound repository macro implementations, external resolved sources,
  and exact item-macro manifests; and
- tightening ratchets.

Once installed, zrail requires no account, daemon, source upload, API key, LLM,
or network access.

## Adopt

For conventional Rust organization, including inline unit tests and Cargo
integration tests:

```sh
zrail init --preset rust
```

The Rust preset does not invent a universal source-file length limit.

For zsumz conventions, including sibling tests, reviewed macro expansion, and
300-line source targets:

```sh
zrail init
```

Both presets write explicit policy to `zrail.toml`; the preset is not hidden
engine behavior.

Plain `init` stops after creating that reviewable contract:

```sh
zrail baseline --dry-run
zrail baseline --accept-grants
zrail check
```

Repository exclusions are applied before Cargo discovery and written exactly
into the contract:

```sh
zrail init --exclude 'fixtures/**' --exclude-from .zrailignore
```

Both flags are repeatable. Exclusion files contain one pattern per line; blank
lines and lines beginning with `#` are ignored, and `!` negation is rejected.
Patterns are normalized, sorted, and deduplicated. An exclusion cannot hide an
active Cargo target. Every exclusion must match inventory except the exact
`target/**` directory of a discovered Cargo workspace or package, which may be
declared before artifacts exist. Once that directory exists, it must contain a
`CACHEDIR.TAG` beginning with the standard signature or the exclusion is
rejected as stale or misdirected.

Add `--baseline` for an atomic contract-and-lock initialization after reviewing
the intended preset and selection:

```sh
zrail init --preset rust --baseline
zrail init --baseline
```

Baseline records supported violations as exact, tightening ratchets. New debt
fails. Cleanup makes the lock stale so it can be tightened. Unsupported
violations abort initialization without leaving partial files.

Ratchets are migration debt, not permanent exemptions. File-size, inline-test,
missing-module-doc, unsafe, and lint-suppression counts are locked to an exact
file and may only decrease. The corresponding strict policy remains active for
every new file; reaching zero makes the contract ratchet stale so it must be
removed rather than retained as dormant authority.

For a hand-authored contract, baseline the existing policy instead of running
`init`:

```sh
# author zrail.toml
zrail check
zrail baseline --dry-run
zrail baseline --accept-grants
zrail check
```

`baseline` preserves comments, existing ratchets, and human-authored reasons.
It can add only registered measurable debt, and it refuses to write while any
other violation remains. Use `--rule rust.file-size` to plan one registered debt
kind, or `--format json` to inspect added, preserved, and rejected candidates.
Grant acceptance is a human review boundary and must not be automated.

## Work

The everyday commands are read-only:

```sh
zrail check
zrail doctor
zrail explain --path src/lib.rs
```

Diagnostic reports use schema 3. Status and the `errors`, `warnings`, `notes`,
and per-rule `groups` counts cover the complete analysis. The `findings` array
contains only the retained individual diagnostics; `summary.retained`,
`summary.omitted`, `truncated`, and `max_findings` describe that payload. The
`analysis` object reports completeness and deterministic workload. Human output
uses the same exact totals.

Individual diagnostics default to 10,000. Use `--max-findings 0` for aggregate-only
output, a non-negative integer for a different bound, or `--max-findings all` for the
complete payload within zrail's existing repository and source safety limits:

```sh
zrail check --max-findings 0 --format json
zrail check --max-findings 50000
zrail check --max-findings all
```

`--max-findings` is accepted by `check` and `review`, whose output contains
diagnostic findings. The former `--limit` spelling remains a deprecated alias.
Doctor and explain reports have no bounded finding payload and reject the
option. Retention never changes pass/fail decisions or protected review
authority; those always use exact totals.

Ordinary repository size is input, not analysis debt. Multiplicative source
contexts and include projection use deterministic input-derived budgets. A
zero-include repository performs zero projection work; active instances are
cached by file and syntax guard, and repeated written-path queries are cached by
source instance, lexical scope, and usage. The qualification suite constructs
10,001 physical Rust files and requires deterministic complete lock state. A
repository may review explicit content-bound overrides when unusual source
shapes need them:

```toml
[analysis.limits]
derived_source_instances = 25000
include_projection_work = 12000000
projected_facts = 500000
```

Budget exhaustion makes analysis incomplete and prevents lock construction.
Display flags never change these analysis budgets.

Zrail's own qualification also rebuilds its lock candidate and rejects growth
above the reviewed `projection_queries` ceiling. `scripts/perf-smoke` repeats
that self-analysis in one warm process and prints advisory wall-clock timing;
elapsed time is operational evidence and never becomes architectural truth.

After committing the initial zrail state, compare later source and policy
changes with the trusted base:

```sh
zrail diff --base HEAD --deny-grants
```

`zrail diff` classifies changes as grants, revocations, debt, cleanup, neutral,
or unknown. `--deny-grants` rejects added power, new debt, and comparisons the
current engine cannot interpret safely.

After human review, update any intentional change to locked architecture state:

```sh
zrail update
```

`zrail update` uses committed `HEAD` architecture as its authority. Use
`--base REVISION` to select another independently reviewed revision.

Policy changes and `zrail update --accept-grants` require explicit human
authorization. Automation should report the semantic diff instead of accepting
added power.

A lock semantic-epoch change is deliberately unknown, not a grant. Reanalyze
the immutable base revision with the new engine and review its scoped report:

```sh
zrail migrate-lock --base HEAD --output zrail-migration.json
zrail update --accept-migration sha256:<reviewed-report-digest>
```

Adapters are explicit and fail closed for unknown epochs. The current engine
can reanalyze locks from every released prior semantics epoch (`1` through `4`)
directly into current semantics `5`; adopters do not need to delete an older
lock or manufacture a lock-free base commit. Each exact old or new authority
subject is classified as preserved, retired, newly observable, or changed
interpretation. Migration acceptance is recomputed for the selected immutable
base and never accepts current-worktree grants; `--accept-grants` remains a
separate authority boundary.

If an engine change makes that immutable base unanalyzable, make only the
required source or contract repairs on a committed descendant and request an
explicit bridge:

```sh
zrail migrate-lock --base <old-good> --target HEAD --output zrail-migration.json
zrail update --base <old-good> --accept-migration sha256:<reviewed-report-digest> \
  --migration-report zrail-migration.json
```

The bridge is available only when strict reanalysis actually fails. The target
must descend from the base, retain the exact prior lock, and pass the current
engine. Its digest binds both commit identifiers, both contract and lock
identities, the normalized base
failure, every added, removed, content-changed, or mode-changed repository file,
and the complete authority-surface comparison. Update recomputes that report,
verifies the named report as the sole nonignored untracked review artifact, and
compares tracked target bytes and modes directly without trusting Git index
flags. Ignored build outputs remain outside the reviewed target state. Any
contract grant remains a separate `--accept-grants` decision.
Gitlinks retain their Git object kind: a target containing any mode-160000
entry is rejected under `submodules = "deny"`, even without `.gitmodules` or
outside source roots. Allowed gitlinks are opaque, untraversed boundaries;
their object identities remain bound by the target commit and change manifest.

### Contract schema and fragments

Schema 2 uses exact fragment paths so large policy registries stay reviewable
without broad filesystem discovery:

```toml
schema = 2
imports = [
  "zrail/policy/dependencies.toml",
  "zrail/policy/ownership.toml",
  "zrail/policy/test-mirrors.toml",
]
```

Imports are repository-relative regular files, may not escape or enter an
excluded path, and fail on cycles, duplicate identities, conflicts, excess
depth, excess bytes, or excess file count. The completeness certificate binds
the normalized path and digest of every loaded source. Schema-1 wildcard
imports remain readable only for migration; schema 2 rejects patterns.

Preview and explicitly apply the deterministic migration, then enforce
preservation-safe TOML validation across the complete fragment bundle:

```sh
zrail migrate-config
zrail migrate-config --write
zrail fmt
zrail fmt --check
```

Migration rewrites `binding` to `resolution`, `bindings` to
`namespace_effect`, and expands legacy wildcard imports to exact paths. It
validates the entire prospective bundle before writing, replaces each source
atomically, refuses to overwrite any source whose bytes changed after planning,
and restores earlier sources if a later replacement fails, reporting any
restoration failure. This is rollback safety, not a crash-atomic
multi-file transaction. Both migration and `fmt` preserve authored comments,
blank lines, ordering, quoting, spacing, and generated markers; `fmt` only
supplies a missing final newline.

## Review

Protected automation can derive a proposal's observed state with a binary and
base revision outside the proposal's control:

```sh
zrail review --base HEAD --authority-root . --root proposal
```

`review` reads proposed Cargo and Rust source as data, verifies its checked-in
lock against an in-memory candidate, and rejects source violations, grants, new
debt, unknown comparisons, and stale or forged locks by default.

`--allow-grants` is an explicit human-review exception for grants. Debt and
unknown comparisons still fail. It must never appear in automated or
proposal-controlled merge checks.

A production deployment needs an organization ruleset workflow stored in a
separately protected repository or a dedicated GitHub App. Require the status
from that exact App and require branches to be current with their base.

Approve an intentional grant through protected authority:

1. Open a policy-only pull request and review its semantic diff.
2. Have a designated owner manually dispatch the separately protected authority
   with the pull request number and explicit grant approval. Bind its status to
   the proposal SHA; debt, unknowns, and source violations remain failures.
3. Merge the policy, rebase the implementation onto it, and require a clean
   automatic `zrail review` with no grant.

This keeps exceptional decisions in protected workflow history instead of
turning `--allow-grants` into a proposal-controlled CI switch.

## Rust

The Rust adapter enforces:

- exact Cargo workspace membership and source-aware dependency edges;
- inspected or reasoned crate roots, including custom `[lib].name` values;
- package layers and allowed dependency direction;
- runtime and compile-time effect boundaries, capability owners, and
  direct-call owners;
- complete traversal within declared repository roots and reviewed generated
  snapshots;
- declarative facades, module contracts, and configurable test placement;
- source hygiene, optional file-size ceilings, and tightening ratchets; and
- invariants connected to exact tests and content-addressed qualification
  gates.

Declarative facades may wire external modules and imports and declare structs,
enums, unions, type aliases, and constants. Implementations, ordinary functions,
statics, inline modules, and other behavioral items remain behind named module
boundaries. A `main` function or procedural-macro entrypoint is declarative only
when its body is empty or a single expression handoff without local statements,
branches, loops, macros, or inline blocks.

External crate-root attestations bind to the exact registry or Git declaration.
Roots that no active canonical policy relies on remain unresolved rather than
being guessed.

Effect profiles evaluate every Cargo target and syntax guard by default. A
runtime boundary can narrow evaluation to ordinary facts reachable from library
or binary targets. Proc-macro implementations have distinct `proc-macro` and
`proc-macro-test` compilation domains and build reachability, not production
reachability. Use `all` to govern their host-side effects as well:

```toml
[profiles.kernel]
reachability = "production"

[profiles.kernel.effects]
deny = ["filesystem", "network", "process"]

[profiles.kernel.syntax]
deny = ["async-fn", "async-block", "async-closure", "await"]
```

Async syntax is independent from the `async-runtime` effect. A runtime-neutral
`async fn`, async block, async closure, or `.await` can therefore be prohibited
without guessing which executor may eventually poll it. Any expansion that the
analyzer cannot inspect directly fails closed unless an exact, provenance-bound
macro allowance separately attests `async_syntax = "none"`; that attestation is
a reviewable grant.

Production reachability excludes integration tests, benchmarks, examples,
build scripts, and facts beneath `#[cfg(test)]`, including guarded code inside a
production-reachable file. `zrail explain` reports both the file's target-domain
reachability and the fact reachability used by each applied profile.

Call and capability owners also evaluate every target and guard by default. Set
`reachability = "production"` on an owner to confine both violations and stale
allowlist checks to runtime-reachable, ordinary facts:

```toml
[[owner]]
name = "process-spawn"
kind = "call"
within = ["crates/kernel/**"]
match = "std::process::Command::new"
allow = ["crates/kernel/src/executor.rs"]
reachability = "production"
reason = "Only the executor may launch child processes at runtime."
```

Directory ownership is repository structural policy and therefore accepts only
the default `all` reachability. `zrail explain` shows each source owner's
effective reachability.

Source-operation owners use the same `within`, `allow`, `reachability`, and
staleness rules as call owners. Type construction is an exact identity rail:

```toml
[[owner]]
name = "state-construction"
kind = "type-construction"
within = ["crates/kernel/src/**"]
match = "crate::state::State"
allow = ["crates/kernel/src/state.rs"]
reason = "State creation stays behind one transition boundary."
```

It observes struct literals, tuple structs, enum variants, and `Self` forms.
Taking a tuple-struct or tuple-variant constructor as a first-class function
value is governed at that acquisition site as constructor capability; the same
`type-construction` owner covers both direct construction and that capability.
Imports and aliases are retained when they resolve exactly. A constructor-like
call that cannot be proven to name a type or variant remains unresolved; an
exact owner fails closed instead of treating capitalization as type evidence.
Type-relative paths such as `<State>::Ready` retain the qualified self type.
An explicit single-item path such as `<State as Factory>::Ready` selects the
trait item and is not construction, even when `State::Ready` is a variant.
If the explicit trait itself cannot be resolved, Zrail retains an unresolved
associated-item boundary instead of fabricating a construction identity.
Longer associated-type projections remain unresolved until their concrete type
identity is proven.
For dependency types whose declaration shape is unavailable, ordinary
`Type::item` syntax stays unresolved even when a local extension trait defines
the same item; only an exact `<Type as LocalTrait>::item` occurrence can prove
that trait-associated value.
Unit struct and enum-variant paths used only as destructuring-assignment
assignees are patterns, not constructions.

Written method-name ownership is deliberately name-level authority:

```toml
[[owner]]
name = "transition-method"
kind = "method-name"
within = ["crates/kernel/src/**"]
match = "transition"
allow = ["crates/kernel/src/state.rs"]
reason = "Every syntactic .transition() call stays centralized."
```

The selector must be one method identifier. It matches every syntactic
`.transition(...)` call regardless of receiver type and does not claim
type-resolved method identity.

Field owners require a qualified field identity. Narrow owners distinguish
reads, writes, and mutable borrows:

```toml
[[owner]]
name = "state-epoch-reader"
kind = "field-read"
within = ["crates/kernel/src/**"]
match = "crate::state::State::epoch"
allow = ["crates/kernel/src/state.rs"]
reason = "Only state queries inspect the epoch."

[[owner]]
name = "state-epoch-writer"
kind = "field-write"
within = ["crates/kernel/src/**"]
match = "crate::state::State::epoch"
allow = ["crates/kernel/src/state.rs"]
reason = "Only state transitions advance the epoch."

[[owner]]
name = "state-epoch-borrower"
kind = "field-mutable-borrow"
within = ["crates/kernel/src/**"]
match = "crate::state::State::epoch"
allow = ["crates/kernel/src/state.rs"]
reason = "Only state transitions borrow the epoch mutably."
```

Reads include ordinary access, immutable borrows, and right-hand-side use.
Assignment, compound-assignment, destructuring-assignment destination leaves,
and mutable address expressions are not also counted as reads. A non-wildcard
field extracted from the right-hand source of a destructuring assignment is a
read, including each projection in a nested assignee; `_` remains ignored.
Functional struct update reads every omitted field known from the local type
declaration. When the declaration's field set is not locally provable, Zrail
retains one unresolved wildcard read that fails closed against every matching
field owner on that source type. Field and initializer `#[cfg]` predicates keep
those implicit reads in their exact compilation worlds. Writes include
ordinary, compound, and destructuring assignment. Mutable borrows include
`&mut value.field`, explicit `ref mut` field patterns, and implicit
mutable-reference bindings introduced by match ergonomics. The same
`field-mutable-borrow` authority category covers `&raw mut value.field`. Calls
such as `mem::replace(&mut value.field, next)` are therefore covered without
guessing that an arbitrary method call mutates its receiver. If a pattern's
effective binding mode cannot be proven, Zrail retains an unresolved mutable
authority candidate instead of describing the access as read-only.

When a field's type exposes mutation through receiver methods, declare those
written method names explicitly:

```toml
[[owner]]
name = "state-entries-mutation"
kind = "field-mutation"
within = ["crates/kernel/src/**"]
match = "crate::state::State::entries"
mutating_methods = ["clear", "insert", "remove"]
allow = ["crates/kernel/src/state.rs"]
reason = "All entry mutation stays behind the state boundary."
```

`field-mutation` combines exact writes and mutable borrows with method calls on
the exact field place only when the written method is in `mutating_methods`.
The list must be sorted and unique. Zrail does not infer mutability from method
names or hardcode methods from standard-library container types.

Use one aggregate owner when the same allow-list governs every form of access:

```toml
[[owner]]
name = "state-epoch-authority"
kind = "field-authority"
within = ["crates/kernel/src/**"]
match = "crate::state::State::epoch"
allow = ["crates/kernel/src/state.rs"]
reason = "All epoch access stays behind the state boundary."
```

`field-authority` combines the same exact read, write, and mutable-borrow facts;
it does not broaden identity resolution. Known `self` receiver types are exact,
while unresolved receiver candidates fail closed for exact field-owner policies.

### Exact type and authority-token rails

Per-type policy selects one declaration by its exact source path and canonical
Rust identity. It can prohibit each derive, manual-implementation, or
macro-output duplication surface independently, or close all modeled Clone/Copy
surfaces together with `clone_copy = "forbidden"`:

```toml
[[source.rust.types]]
name = "command-ticket"
match = "crate::command::Ticket"
path = "crates/kernel/src/command.rs"
kind = "type"
reachability = "production"
deny = ["derive-clone", "derive-copy", "impl-clone", "impl-copy", "opaque-expansion"]
reason = "Tickets transfer command authority and must not be duplicated."
```

Use either the individual `deny` list or `clone_copy = "forbidden"`, never
both on the same policy. The bundle closes exactly the five modeled
prohibitions shown above. Semantic diffs compare that effective prohibition
set, so changing between the bundle and the complete expanded list is an
authority-neutral normalization. An `authority-token` intentionally requires
the bundle as its explicit Clone/Copy-closed intent marker.

Derive and manual-implementation checks cover structs and enums. Manual
implementations use the ordinary canonical identity layer, so aliases such as
`use std::clone::Clone as C; impl C for Ticket` and qualified
`impl core::marker::Copy for Ticket` are not different escape hatches.
Unresolved trait or type identities fail closed rather than matching by their
last path segment. A written `Clone` or `Copy` derive also remains a violation
if its compiler-builtin provenance becomes unresolved.

Forbidden Clone/Copy closes macro output across the selected type's whole active
package and compilation world. An item or attribute macro does not need to
overlap or mention the declaration to be capable of emitting a duplication
implementation. The expansion is closed only when the compiler understands it
directly or an exact, immutable macro allowance separately attests:

```toml
[[source.rust.macros.allow]]
name = "ticket_macro::metadata"
resolution = "exact"
inputs = "inspect"
namespace_effect = "opaque"
async_syntax = "opaque"
duplication_effect = "none"
reason = "Reviewed expansion cannot add Clone or Copy implementations."

[source.rust.macros.allow.source]
kind = "cargo-lock"
package = "ticket-macro"
```

`duplication_effect = "none"` is a provenance claim, not a name allowance. It
requires exact binding plus an immutable external source or exact repository
definition. A same-spelling, ambiguous, conditionally different, or unbound
macro remains opaque.

Authority tokens add an exact representation contract:

```toml
[[source.rust.types]]
name = "leadership-permit"
match = "crate::authority::LeadershipPermit"
path = "crates/kernel/src/authority.rs"
kind = "authority-token"
clone_copy = "forbidden"
visibility = "private"
leaf_module = true
reason = "The permit is private, shape-bound leadership authority."

[[source.rust.types.fields]]
name = "epoch"
type = "u64"
visibility = "private"

[[source.rust.types.fields]]
name = "owner"
type = "core::option::Option<crate::node::NodeId>"
visibility = "private"
```

An authority-token policy must forbid Clone/Copy, require private type
visibility, a leaf module, an exact ordered field list, and private visibility
on every field. Field names, order, visibility, and complete semantic types are
part of the representation authority. This is a Clone/Copy-closed shape
contract, not a claim that Rust enforces linear or one-use values: ordinary
construction, manual reconstruction from fields, and dropping remain separate
properties. Use type-construction ownership when minting must also be confined.
Supported exact types include qualified paths
with recursive generic arguments, tuples, references, slices, arrays, raw
pointers, unit, and never. Non-primitive paths at every nesting level must be
qualified; `impl Trait`, `dyn Trait`, inference, type macros, bare function
types, and associated or constrained generic arguments are rejected rather
than truncated to an outer path. An authored leading `crate::` is normalized
against the analyzer's canonical local identity.
Array lengths and simple braced const generic paths resolve in Rust's value
namespace, independently of type names. Arbitrary const expressions and blocks
with statements remain unsupported.

Exact shape is resolved separately in every governed Cargo compilation domain.
Inactive fields and child modules are absent; possible target predicates make
their shape unresolved. Every selected feature world and test mode must match
the contract, and mismatch diagnostics identify the domain. Coverage reports
one resolved declaration shape per logical source occurrence in each domain,
not a union of written fields. Leafness includes child modules introduced in
parent and sibling item includes. Repeated mounts retain separate leafness;
diagnostics identify the source occurrence. An opaque logical module namespace
cannot prove leafness: `duplication_effect = "none"` alone is insufficient,
while occurrence-exact `namespace_effect = "none"` can close that boundary.
An active or possibly active item-replacing attribute makes exact shape
unsupported, even with `namespace_effect = "none"`: namespace preservation
does not attest preservation of fields or representation. No shape-preservation
macro grant is currently supported.

Repository-wide written syntax is intentionally separate from per-type
Clone/Copy closure:

```toml
[source.rust.duplication]
reachability = "production"
deny_imports = ["clone", "copy"]
deny_macro_tokens = ["clone", "copy"]
```

These settings reject explicit imports or aliases and matching identifiers in
opaque macro token streams throughout the selected reachability. They do not
silently broaden a named type policy. `zrail coverage --format json` reports
both global syntax occurrences and every type policy's authored shape plus its
actual declaration, derive, manual-implementation, and opaque-expansion facts,
including compilation worlds and allowed or closed status.

Rust file roles are inferred from conventional paths. An exceptional source may
be reclassified only between `facade` and `implementation` with an exact path
and a durable reason:

```toml
[[source.rust.file_roles]]
path = "crates/raft-log/src/raft_log_segment.rs"
role = "facade"
reason = "Reviewed public surface; implementation belongs behind child modules."
```

The effective role drives both declarative-shape enforcement and size budgets.
Overrides for missing, unreachable, generated, already matching, test, or
auxiliary source fail as stale or invalid policy. An entrypoint may be
reclassified only as `implementation`; treating it as a facade is invalid.
`zrail explain` shows the inferred role, effective role, and override reason.

Written glob imports have a separate closed hygiene policy; name resolution
continues to resolve globs regardless of this setting:

```toml
[source.rust.hygiene]
glob_imports = "facade-reexports-only"
```

`facade-reexports-only` permits only a top-level outward `pub use path::*` in an
effective facade. `facade-reexports-and-test-super` additionally permits the
exact private import `use super::*` when its source-graph reachability or syntax
guard proves it is test-only. Outside the existing outward-facade allowance, it
does not allow production `super::*`, dependency globs, public re-exports, or
other test globs. The remaining modes are `allow` and `deny`.

Conventional `tests.rs`, `*_test.rs`, and `*_tests.rs` filenames receive test
tooling defaults and budgets, as does source beneath a `tests/` directory.
`test.rs` remains an ordinary implementation filename; source-graph
reachability independently determines whether any file is production or test
reachable.

## Generated inputs

Reviewed generated trees and `OUT_DIR` snapshots bind their source, generator,
and declared inputs. A changed or missing input makes the lock stale.

`env!` and `option_env!` provide the `compile-environment` effect. `include!`,
`include_str!`, and `include_bytes!` provide `compile-filesystem`; literal file
inputs must resolve to inventoried files inside the repository. These are
separate from runtime `environment` and `filesystem` effects.

## Macros

With `source.rust.macros.mode = "deny-unreviewed"`, expansion boundaries that
zrail cannot inspect are rejected unless their invocation path has a reasoned
allowance. Ordinary Rust expressions inside standard macro inputs are still
analyzed. Other token DSLs require the separate `inputs = "opaque"` grant.

Repository-owned macros lock a deterministic input digest of their implementing
package, including helper macros and internal proc macros. Input capture follows
the bounded transitive closure of workspace-member and repository-path helper
crates, binding every owned regular file (including JSON, templates, manifests,
and Rust), the workspace manifest, any Cargo lock, and literal compile inputs.
Any registry or Git dependency in that provider/helper closure requires a
regular `Cargo.lock`. Each closure manifest edge must resolve to exactly one
outgoing lock edge; the validated entire lock binds concrete package names,
versions, sources, checksums, and transitive dependency edges. Local-only
closures can remain lock-free.
Source exclusions cannot hide provider inputs. Missing internal packages or
inputs, symlinks, unresolved includes, and exceeded input limits fail closed.
Capture traverses only provider/helper roots and fixed prefixes of declared
input patterns, not unrelated excluded repository trees. Reserved `.git`,
`.zrail`, and `target` components are pruned before descent at every depth.
External allowances bind to the exact
dependency source. Built-in data macros and `include!` are handled directly,
and included Rust remains fully analyzed.

Cross-crate workspace macros can bind their observed implementation directly:

```toml
[[source.rust.macros.allow]]
name = "workspace_macros::reviewed"
reason = "Reviewed workspace expansion."

[source.rust.macros.allow.source]
kind = "repository"
package = "workspace-macros"
directory = "crates/workspace-macros"
inputs = ["schemas/api.json"]
ambient_inputs = "none"
```

The package and directory must match the resolved repository origin. Optional
`inputs` patterns are repository-relative additions to the owned input set;
each pattern must match a regular file. Capture is bounded to 4,096 files and
64 MiB of framed input per implementation, plus individual-file read limits.
The lock stores the deterministic result as `inputs_sha256`; another package
exporting the same macro name cannot borrow this authority. A qualified name
alone is insufficient: repository allowances require this source or an exact
local `definition`.

`ambient_inputs = "none"` is mandatory for repository source authority. It is
an explicit, source-bound review attestation that macro output depends only on
invocation tokens and captured inputs, not unbound environment values, build
outputs, filesystem contents or metadata, process results, or network inputs.
`CARGO_MANIFEST_DIR` may locate the bound package tree, but its absolute value
must not influence output. `.git`, `.zrail`, `target`, and `zrail.lock` paths are
reserved and excluded from capture; reliance on them violates this attestation.
Keep any custom output/lock paths outside the captured package tree. This is
not a sandbox or static proof of hermeticity: a provider that needs undeclared
ambient inputs cannot honestly receive this authority. Changed provider/helper
files or reviewed input patterns require review of the assumption again.

Ordinary macro permission does not make expansion output visible. Attribute,
derive, and item macros therefore keep the surrounding namespace opaque unless
an exact occurrence bound to `source` or `definition` provenance separately sets
`namespace_effect = "none"`. That grant attests zero ordinary-namespace delta: the
reviewed expansion preserves the annotated item subtree with the same binding
kind, target, visibility, cfg domain, and child namespace, and introduces no
surrounding lexical bindings. External attestations require an exact registry
version or full Git object ID. The attestation is applied only to occurrences
whose complete resolved origin matches it; a same-spelling macro from another
source or conditional definition remains opaque and fails binding review.

Source-operation owners fail closed across the same expansion boundary. Every
opaque function-like, statement, item, derive, or attribute macro invocation
inside a `type-construction`, `method-name`, or field-operation owner's `within`
scope is an unresolved candidate for that owner. It produces `OWN-003` outside
the allowed owner files and `OWN-006` inside them; ordinary macro permission
alone cannot make the unknown operation exact. Exact review may close only this
boundary with the separate provenance-bound claim:

```toml
[[source.rust.macros.allow]]
name = "model_macro::metadata"
resolution = "exact"
source_operations = "none"
reason = "Reviewed expansion constructs no types and performs no field or method operations."

[source.rust.macros.allow.source]
kind = "cargo-lock"
package = "model-macro"
```

`source_operations = "none"` defaults to `"opaque"` and is a grant in semantic
diffs. It closes only occurrences whose observed resolution is exact and whose
provenance is an immutable dependency source, a content-locked repository
source, an exact repository definition, or a sole exact compiler-builtin
origin. An allowance may remain
`resolution = "conservative"` for unresolved sites with the same spelling;
those sites stay opaque and do not downgrade exact occurrences. Compiler
expansions whose output Zrail directly inspects are already closed.

Field mutation can be reviewed without making the broader and usually false
claim that an expansion performs no source operations:

```toml
[[source.rust.macros.allow]]
name = "workspace_macros::metadata"
resolution = "exact"
field_mutation = "none"
reason = "Reviewed expansion writes, mutably borrows, and mutates no governed field."

[source.rust.macros.allow.source]
kind = "repository"
package = "workspace-macros"
directory = "crates/workspace-macros"
ambient_inputs = "none"
```

This closes only `field-write`, `field-mutable-borrow`, and `field-mutation`
owners. Field reads, full field authority, construction, and method-name owners
remain opaque unless the stronger `source_operations = "none"` claim applies.

Literal and verified generated `include!` sources retain occurrence-specific
lexical splices. Textual `macro_rules!` lookup follows caller prefixes, nested
includes, source order, lexical scope, and Cargo compilation domains; item
includes can introduce definitions into the caller suffix, while expression
includes cannot leak definitions beyond their expression scope. Cross-file
`use` aliases are conservatively unresolved when they could change a macro
invocation across a splice. Until that import projection is exact,
the invocation requires an explicit name-only `resolution = "conservative"`
allowance and cannot borrow compiler or dependency-source authority.

Ordinary paths and direct calls use the same occurrence-specific namespaces.
Explicit imports, glob imports, type aliases, module re-exports, visibility,
Rust editions, lexical scopes, `cfg(test)` domains, and nested or repeated
includes are projected onto every applicable source instance. A projected call
remains exact only when every instance resolves to the same identity;
disagreement is conservative, and incomplete or macro-opaque lookup is
unresolved. Call ownership therefore cannot accept a physical-file spelling
that the effective Rust namespace resolves to an owned call.

Qualified `self`, `crate`, and `super` paths navigate the effective module
before binding lookup; include edges do not create module boundaries. The
repository plans all ordinary namespace projections transactionally under shared
work and fact budgets, so exhaustion emits one unresolved diagnostic and
retains no partial authority result.

### Exact Cargo feature worlds

By default, zrail preserves its legacy conditional model: feature-gated source
is retained conservatively, and the analysis is not described as an exact Cargo
feature compilation. Repositories that need exact feature-specific policy can
declare named, workspace-wide worlds:

```toml
[[source.rust.feature_worlds]]
name = "strict"
reason = "The supported strict product build."

[[source.rust.feature_worlds.packages]]
package = "kernel"
default_features = false
features = ["strict"]

[[source.rust.feature_worlds.packages]]
package = "protocol"
default_features = true
features = []
```

Every world must select every active workspace package exactly once. Those
selections describe zrail's workspace-wide static source invocation across all
discovered targets; they do not model one particular `cargo -p` command, target
triple, or root selection. The accepted subset guarantees one feature set per
package across target versus host, normal versus build or proc-macro, and
ordinary versus test, example, or benchmark contexts.

Zrail validates declared features, default-feature closure,
optional-dependency activation, dependency feature forwarding, and workspace
dependency propagation to a fixed point without invoking Cargo. It compares a
lower closure that excludes target-conditional, build, and development edges
plus every internal edge downstream of those contexts or a proc-macro host
package with an upper closure that includes them. Split-context reachability is
propagated transitively through the bounded workspace package graph, including
through optional edges that are inactive in the authored world. A difference
rejects the world as inexact and names its exact feature and dependency witness.
Equality of the two global unions is not sufficient: every structurally
context-split package must also have an empty upper active feature set.

This is a deliberately severe precision cost for avoiding a Cargo
compilation-unit graph. Zrail rejects a featureful split-context package even
when Cargo would give its host and target units the same features, and even
when resolver 1 would unify them. Feature-empty split subgraphs are accepted.
Proc-macro hosts are recognized from both `[lib] proc-macro = true` and an
explicit `[lib] crate-type = ["proc-macro"]`; Cargo's explicit `crate-type`
list takes precedence over the boolean.

The convergence requirement applies to Cargo resolver 1, 2, and 3 workspaces.
Zrail does not currently model the resolver version or a Cargo compilation-unit
graph. Resolver 1, 2, and 3 repositories all use the same feature-empty
split-context proof boundary. Cargo-backed resolver comparisons live only in
zrail's trusted test fixtures; the runtime analyzer never invokes Cargo.

Each resulting Cargo compilation domain includes the world name and the exact
active features for its package. `cfg(feature = "...")`, Boolean combinations,
and nested `cfg_attr(..., cfg(...))` are reduced exactly per world. Other target
predicates such as `cfg(unix)` remain conservative. A feature-dependent
`cfg_attr` that changes `path`, `test`, or `bench` identity fails completeness,
because selecting source or a Cargo test target requires evidence beyond a
syntax-only feature reduction. Cargo target `required-features` are parsed and
targets missing those features are not seeded in that world.

The lock certificate binds package feature definitions, target
`required-features`, configured selections, resolved closures, and world count
separately from the general inventory. Changing any of them makes the lock
stale. `zrail coverage` reports every configured world and includes the world
and active feature set on each compilation-domain occurrence.

An optional `definition` path can narrow a `macro_rules!` allowance, but path
spelling never establishes origin. The default `resolution = "exact"` rejects an
allowance when the candidate origin remains unresolved. A name-only allowance
may opt into `resolution = "conservative"` to cover only the exact spelling at the
invocation site; it cannot claim a `source` or `definition` for that unresolved
candidate. Repository globs are narrowed against the bounded local macro
namespace, while ambiguous glob candidates must all be allowed. `#[macro_use]`
imports remain unresolved because their bare namespace cannot be attributed
exactly without compiler expansion.

`resolution` is the maximum fallback a name allowance permits, not a quality
assigned to every use of that name. Each invocation retains its own exact,
conservative, or unresolved result. Closed output claims apply only to exact
occurrences, so one include-mounted or shadowed spelling cannot downgrade—or
lend authority to—other occurrences. `zrail explain` renders that per-occurrence
quality, source span, and invocation-input digest.

Macro policy names are user-spellable Rust paths. Diagnostics prefer the stable
public path (`quote::quote`) while an exact lexical spelling (`q`) may satisfy
the same single resolved candidate. Dependency package and source provenance
remain separate authority in `source`; zrail never encodes provenance by
repeating path segments. `zrail explain` lists each observed macro's written
spelling, preferred policy name, and resolved origin independently.

Item-producing macros are a separate source-graph boundary because their output
can declare modules or include source that static traversal cannot see. Existing
exact-path authority remains valid, while repeated harness macros can be scoped
by name and repository glob:

```toml
[[source.rust.item_macros]]
name = "criterion_group"
within = ["benches/**"]
reason = "Reviewed benchmark harness emits no source edges."
```

Omitting both `path` and `within` grants repository-wide name authority. A
contract may use either an exact `path` or `within`, never both. Exact paths go
stale when their invocation disappears; scoped and repository-wide entries go
stale only when no reachable invocation remains inside their authority.

Name matching alone makes no provenance claim. Set `resolution = "exact"` to
require the same fail-closed macro-origin resolution used by expansion policy;
an external `source` is accepted only with that explicit exact binding.
`resolution = "conservative"` can cover an unresolved exact spelling but cannot
claim external provenance. `zrail explain` identifies the item-macro entry that
actually authorizes each explained file.

External exact binding also requires the dependency's Rust crate root to be
known from Cargo or a matching `dependencies.crate_root` attestation.

```toml
resolution = "exact"

[source.rust.item_macros.source]
kind = "registry"
requirement = "0.5"
```

### Exact item-macro namespace manifests

Permitting an item-position macro does not pretend its generated names are
known. When those names affect ordinary path resolution, bind one exact
invocation to a checked-in namespace manifest:

```toml
[[source.rust.item_macros]]
name = "declare_states"
path = "src/state.rs"
resolution = "exact"
manifest = "zrail/macros/declare-states.toml"
reason = "Reviewed declarations are part of the static namespace."
```

```toml
schema = 1
macro_name = "declare_states"
invocation_sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[[binding]]
name = "Ready"
kind = "constructor"
public = true
```

The exact source path must contain one matching invocation. The manifest token
digest must match its canonical input, and binding names must be unique Rust
identifiers with an explicit type, constructor, or value namespace kind. Those
declarations enter normal path resolution only after validation. The lock binds
the manifest path and bytes, invocation digest, exact resolved macro definition
and content digest, effective syntax guard, every applicable Cargo compilation
domain, and binding count. Repository definitions bind their canonical macro
token stream; external definitions require an exact Cargo.lock package with an
archive checksum. Source, manifest, guard, domain, or namespace drift therefore
requires review, and ambiguous definitions fail closed.

### Exact test mirrors and execution receipts

An exact test mirror pairs one production Rust file with one named test in one
Cargo-test-reachable Rust file. This release deliberately uses explicit pairs;
it does not infer mirrors from file names or expand path templates.

```toml
[[source.rust.test_mirrors]]
production = "src/state.rs"
test = "tests/state_test.rs"
name = "state_transitions"
receipt = "evidence/state-transitions.json"
inputs = [
  "Cargo.lock",
  "Cargo.toml",
  "fixtures/state-cases.json",
  "src/model.rs",
]
reason = "The test exercises state transitions through the public surface."

[source.rust.test_mirrors.execution]
command = "cargo test --package kernel --test state_test state_transitions --target x86_64-unknown-linux-gnu"
package = "kernel"
default_features = false
features = ["strict"]
target = "x86_64-unknown-linux-gnu"
toolchain = "rustc 1.90.0 (example 2026-01-01)"
```

The production path must exist and be reachable from a Cargo production target.
The test path must exist, be classified as test source, be reachable from a
Cargo test target, and declare the exact named `#[test]` once. Production paths,
test paths, and receipt paths are each unique across the mirror set, so a file
or receipt cannot be reused and a removed path becomes stale policy. `execution`
is a required nested table with a closed field set; unknown mirror or execution
keys, missing fields, and duplicate fields fail closed. Inputs are unique,
sorted repository paths and must include `Cargo.toml`, `Cargo.lock`, and the
selected package manifest. The selected package must own both source files.

Mirror execution features are checked against source availability. With
configured feature worlds, the execution package's `default_features` and
exact selected `features` must identify one and only one world; zero or multiple
matches fail closed. The named test must be present exactly in that world and
its Cargo target's `required-features` must be active. Without configured
worlds, the explicit execution identity supplies the legacy mirror's local
feature closure. Zrail still validates this statically and does not run Cargo.

zrail does not execute tests. A test runner records execution in strict schema-2
JSON after the exact test passes:

```json
{
  "schema": 2,
  "producer": "test-runner 1.2.3",
  "input_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "execution": {
    "command": "cargo test --package kernel --test state_test state_transitions --target x86_64-unknown-linux-gnu",
    "package": "kernel",
    "default_features": false,
    "features": ["strict"],
    "target": "x86_64-unknown-linux-gnu",
    "toolchain": "rustc 1.90.0 (example 2026-01-01)"
  },
  "tests": [{ "id": "state_transitions", "status": "passed" }]
}
```

`producer` must contain a name and `major.minor.patch` version. The requested
test must appear exactly once with status `passed`; failed, skipped, missing, or
duplicate outcomes fail closed. The receipt execution object must exactly equal
the contract. `input_sha256` hashes the domain
`zrail-test-mirror-input-v2\0`, followed by length-framed production path and
bytes, test path and bytes, test name, the count and sorted path/bytes for every
reviewed input, then command, package, default-feature flag, feature count and
features, target, and toolchain. Every field uses an unsigned 64-bit big-endian
byte-length prefix. Missing inputs, package/source mismatches, context drift,
and digest drift fail closed. The lock additionally hashes the exact receipt
bytes and retains the mirror identity, declared input digest, and versioned
producer, so replacing a still-valid receipt remains explicit lock drift.
The reviewed input list is explicit rather than inferred: shared modules,
fixtures, build scripts, and generated inputs that affect the test must be
listed. An omitted dynamic input is not claimed as attested by this receipt.

For large mirror sets, `zrail mirrors plan --format json` emits a strict,
digest-bound execution plan. It hashes each unique reviewed input only once,
retains every mirror's independent digest, validates exact source reachability
and test declarations, and groups identical execution identities without
merging their receipt authority. A separately trusted producer can consume the
plan and execute each group. It returns one strict result object per policy,
grouped by the plan's exact execution-group digest and bound to the plan digest.
Coverage and plan output use the same `test-mirror:sha256:<digest>` policy ID.
That digest is domain-separated and length-frames the exact production path,
test path, and test name, so delimiter-bearing paths cannot collide.

```json
{
  "schema": 1,
  "plan_sha256": "<exact plan digest>",
  "producer": "trusted-runner 1.2.3",
  "groups": [{
    "execution_group": "<group digest from the plan>",
    "tests": [{"policy_id": "<policy id from the plan>", "status": "passed"}]
  }]
}
```

`zrail mirrors receipts --plan evidence/mirror-plan.json --results
evidence/mirror-results.json --format json` rejects missing, extra, duplicate,
mis-grouped, stale, or non-canonical results and renders every schema-2 receipt
in one deterministic bundle. Each artifact contains the exact newline-terminated
JSON `source`, its SHA-256, and its declared repository path; the trusted
producer writes those bytes without reserializing them. Then
`zrail mirrors verify --plan evidence/mirror-plan.json` recomputes the complete
plan from current repository bytes before checking only the declared mirror
receipts. Zrail still does not execute repository programs or infer that a
passing command ran an unreported test. Failed and skipped outcomes can be
recorded honestly, but they do not satisfy mirror verification.

## Cargo

Contract parsing is strict. Unknown keys, stale policy, unresolved source
boundaries, missing evidence, and unreviewed lock changes fail closed.

Cargo package and source analysis follows the active root workspace. Declared
members and in-repository path dependencies are included; unrelated or excluded
nested workspaces remain separate repository boundaries. An active path
dependency that crosses into a nested workspace fails explicitly because zrail
does not guess across multiple workspace inheritance roots.

Dependency prohibitions are direct by default. Set `reachability =
"transitive"` to check every package reachable through the checked-in
`Cargo.lock`, and optionally select the kind of the first manifest edge:

```toml
[[dependency]]
name = "runtime-supply-chain"
from = "service"
deny = ["blocked-package"]
reachability = "transitive"
kinds = ["normal", "build"]
reason = "The service may not reach the prohibited package at runtime or build time."
```

Resolved findings report the shortest path and every node's exact package name,
version, source, and checksum. Multiple locked versions remain distinct. A
manifest declaration that maps to zero or multiple outgoing lock nodes fails
closed instead of guessing. Cargo records dependency kinds in manifests but not
on downstream `Cargo.lock` edges, so `kinds` honestly selects only the first
edge leaving `from`; the remainder of the path is resolved without an invented
kind. Transitive policy requires a checked-in lock file.

An immutable external macro authority may select one resolved lock node. Add
`version` or `source` whenever the package name alone is not unique:

```toml
[[source.rust.macros.allow]]
name = "codegen_macro::generate"
resolution = "exact"
namespace_effect = "none"
reason = "Reviewed expansion preserves the surrounding namespace."

[source.rust.macros.allow.source]
kind = "cargo-lock"
package = "codegen-macro"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
```

The selector is authority only when it identifies exactly one lock node. The
resolved version, source, and checksum are then available for lock-state
binding; ambiguous selectors are rejected.

Cargo permits `rev` to name either a commit or a remote named reference. Zrail
intentionally accepts only hexadecimal commit prefixes whose selected precise
commit is proven by `Cargo.lock`. Named `rev` values such as pull-request refs
fail closed; use an exact branch, tag, or hexadecimal revision when declaring a
Git dependency governed by Zrail.

Repository-controlled Cargo source overrides and registry mappings are rejected
until zrail can attest their effective resolution. Root `.cargo/config` and
`.cargo/config.toml` are rejected because Cargo can use them to alter dependency
resolution and qualification execution.

## Coverage audit

`zrail coverage --format json` emits schema-versioned, deterministic evidence
for the governed surface without consulting the lock state as policy authority
or modifying the repository. The report includes the complete analysis work
census and exact repository exclusions; every enabled owner policy with its
canonical ID, selector, scope, allow list, and matched source-operation
occurrences; and every dependency prohibition with exact shortest violating
paths through `Cargo.lock` identities. Occurrences retain source spans,
resolution quality, syntax guards, applicable Cargo compilation domains, and
whether the source path is allowed. Runtime-neutral syntax and written glob
policies additionally retain exact occurrences, visibility, and lexical scope.
Unresolved and conservative matches are counted explicitly. Schema 5 binds the
exact resolved contract digest and also includes `enabled_rails`, a sorted canonical
census covering every global policy switch, contract source, analysis limit,
and named layer, scope, owner, ratchet, gate, invariant, macro, generated-source,
dependency, and test-mirror rail. Exact test-mirror identities include their
reviewed inputs, command, package, feature set, target, and toolchain. The report
also lists each exact feature world and the world plus active feature set on
every applicable compilation domain.

Coverage is an audit artifact, not partial best-effort discovery. It fails when
source analysis is incomplete, when a governed dependency cannot be mapped to
one exact outgoing lock node, or when dependency prohibitions exist without a
checked-in `Cargo.lock`. The human format summarizes the same complete model;
JSON is the stable input for external coverage tooling.

## Commands

| Command | Purpose |
| --- | --- |
| `zrail init` | Write explicit policy, optionally with an atomic initial baseline and lock |
| `zrail baseline` | Add reviewed tightening ratchets to an existing contract |
| `zrail check` | Check repository architecture without modifying files |
| `zrail coverage` | Export the complete governed surface for audit tooling |
| `zrail mirrors plan` | Emit an exact grouped plan for a separately trusted receipt producer |
| `zrail mirrors receipts` | Render all schema-2 receipts from strict plan-bound producer results |
| `zrail mirrors verify` | Recompute a plan and verify its exact schema-2 receipts |
| `zrail doctor` | Diagnose setup and compatibility problems |
| `zrail explain` | Explain the policy and findings for one path |
| `zrail diff` | Classify architecture changes between trusted states |
| `zrail fmt` | Validate exact contract TOML without erasing authored layout or comments |
| `zrail migrate-config` | Preview or apply the schema-1 to schema-2 contract migration |
| `zrail migrate-lock` | Reanalyze an immutable base or review an explicit descendant migration bridge |
| `zrail update` | Refresh reviewed lock state from committed authority |
| `zrail review` | Analyze an untrusted proposal from protected authority |

Use `zrail <command> --help` for the exact options accepted by the installed
version. Human-readable and JSON output are available across the reporting
commands.

## Qualification

`QUAL-01` connects reviewed local and hosted gates to exact test evidence. The
lock hashes each gate and its declared behavioral inputs, including helper
scripts, workflow actions, workspace manifests, the Rust toolchain, formatting
and lint configuration, and ignore rules.

Canonical qualification uses Rust 1.97.1. A compile-only job checks all targets
and features with Rust 1.96.1, preserving the Rust 1.96 source compatibility
floor without qualifying releases on the older compiler.

Release tags use a separate protected workflow. Repository rules must restrict
`v*` tags, and the `release` environment must require review and allow only
those protected tags. Before repository code runs, the workflow proves that the
tagged commit is reachable from the protected default branch and that the tag
version matches `Cargo.toml`. It then runs the complete qualification gate,
builds all seven targets with the pinned toolchain and lockfile, smoke-checks
every binary, and exercises Linux archives in clean containers without Rust.
Tags may be stable `vX.Y.Z` releases or SemVer prereleases such as numbered
`vX.Y.Z-rc.N` candidates. Prerelease tags bind `prerelease = true` into the
reviewed GitHub identity.

No release is visible until every target and clean-runtime check succeeds. The
publisher also packages and verifies `zrail-core`, `zrail-rust`, and `zrail`
from the same reviewed checkout. It requires the exact binary and crate set,
records every digest in `SHA256SUMS`, and creates GitHub build-provenance
attestations. Packaging safely stages each predecessor archive in an offline,
versioned Cargo directory source: every member must be a regular file or
directory beneath the exact package root, and Cargo's checksum manifest binds
both the archive and every extracted file. Dependent archives must retain the
canonical crates.io source and the attested predecessor checksum in their
shipped lockfile; that lockfile is checked directly and is never replaced.
Before opening a draft or publishing a crate, the publish job installs the same
pinned Rust toolchain, regenerates all three archives in
`cargo publish --dry-run` mode against the same kind of offline staged source,
and requires every byte to match its attested input. A temporary crates.io
trusted-publishing token then publishes the crates in dependency order without
source replacement or executing package code; each registry archive is
downloaded and compared byte-for-byte again. A repository-owned API client
recovers drafts through their GraphQL database ID and fetches complete release
state through REST; it does not use the published-release by-tag endpoint or
`target_commitish` as authority. The client resolves the remote tag reference,
peels annotated tags to the qualified commit, and binds draft state,
prerelease state, title, exact reviewed notes, asset names, and every asset byte.
A rerun rejects unexpected or altered state and uploads only missing expected
assets. An expected asset left in GitHub's documented `starter` state is deleted
by its numeric asset identity, refetched as missing, and reuploaded; uploaded
assets with different bytes and all unknown asset states fail closed. An exact
already-published release is also a successful retry state, but it does not skip
the intervening registry-archive verification. The mocked release rehearsal
injects both a `starter` residue and a lost successful publish response, then
proves that the same release resumes without replacing reviewed bytes or
publishing twice.
Only after all three registry comparisons pass does the workflow make the
GitHub draft visible. Release actions are pinned to full commit identities; the
workflow does not accept proposal or manual source inputs.

For a protected deployment, proposed checker changes must not authorize
violations or grants in the same pull request. The required result must come
from a ruleset workflow or App outside the proposal's write domain.

Run the complete offline repository gate with:

```sh
scripts/check
```

It checks structure, formatting, Clippy, tests, rustdoc, zrail itself, package
archives, whitespace, and that Git status is unchanged by the gate.

## Limits

zrail currently analyzes Rust and Cargo repositories. It evaluates declared
source and reviewed snapshots as data; it does not execute repository code,
Cargo, build scripts, gates, or generated programs.
