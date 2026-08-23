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

- normalized direct dependency declarations;
- reviewed gate bytes and their declared behavioral inputs;
- generated-source provenance;
- content-bound repository macro implementation packages; and
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

Add `--baseline` when adopting an existing repository:

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

Diagnostic reports use schema 2. Status and the `errors`, `warnings`, `notes`,
and per-rule `groups` counts cover the complete analysis. The `findings` array
contains only the retained individual diagnostics; `summary.retained`,
`summary.omitted`, `truncated`, and `limit` describe that payload. Human output
uses the same exact totals.

Individual diagnostics default to 10,000. Use `--limit 0` for aggregate-only
output, a non-negative integer for a different bound, or `--limit all` for the
complete payload within zrail's existing repository and source safety limits:

```sh
zrail check --limit 0 --format json
zrail check --limit 50000
zrail check --limit all
```

`--limit` is accepted by `check` and `review`, whose output contains diagnostic
findings. Doctor and explain reports have no bounded finding payload and reject
the option. Retention never changes pass/fail decisions or protected review
authority; those always use exact totals.

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

A lock semantic-epoch change is deliberately unknown, not a grant. Repository
governance should require a separately reviewed, signed bootstrap because the
older protected engine cannot interpret the newer state. The CLI does not
enforce signing policy.

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

The included repository workflow demonstrates protected-base analysis. It is
useful preview feedback, but it is not production merge authority: a proposal
can add another workflow under the same GitHub Actions identity.

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

External crate-root attestations bind to the exact registry or Git declaration.
Roots that no active canonical policy relies on remain unresolved rather than
being guessed.

Effect profiles evaluate every Cargo target and syntax guard by default. A
runtime boundary can narrow evaluation to ordinary facts reachable from library
or binary targets:

```toml
[profiles.kernel]
reachability = "production"

[profiles.kernel.effects]
deny = ["filesystem", "network", "process"]
```

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
Overrides for missing, unreachable, generated, already matching, test,
entrypoint, or auxiliary source fail as stale or invalid policy. `zrail explain`
shows the inferred role, effective role, and override reason.

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

Repository-owned macros lock a deterministic manifest of their implementing
package, including helper macros and internal proc macros. External allowances
bind to the exact dependency source. Built-in data macros and `include!` are
handled directly, and included Rust remains fully analyzed.

Literal and verified generated `include!` sources retain occurrence-specific
lexical splices. Textual `macro_rules!` lookup follows caller prefixes, nested
includes, source order, lexical scope, and Cargo compilation domains; item
includes can introduce definitions into the caller suffix, while expression
includes cannot leak definitions beyond their expression scope. Cross-file
`use` aliases are conservatively unresolved when they could change a macro
invocation across a splice. Until that import projection is exact,
the invocation requires an explicit name-only `binding = "conservative"`
allowance and cannot borrow compiler or dependency-source authority.

Ordinary paths and direct calls use the same occurrence-specific namespaces.
Explicit imports, glob imports, type aliases, lexical scopes, `cfg(test)`
domains, and nested or repeated includes are projected onto every applicable
source instance. A projected call remains exact only when every instance
resolves to the same identity; disagreement is conservative, and incomplete
lookup is unresolved. Call ownership therefore cannot accept a physical-file
spelling that an include-spliced binding resolves to an owned call.

An optional `definition` path can narrow a `macro_rules!` allowance, but path
spelling never establishes origin. The default `binding = "exact"` rejects an
allowance when the candidate origin remains unresolved. A name-only allowance
may opt into `binding = "conservative"` to cover only the exact spelling at the
invocation site; it cannot claim a `source` or `definition` for that unresolved
candidate. Repository globs are narrowed against the bounded local macro
namespace, while ambiguous glob candidates must all be allowed. `#[macro_use]`
imports remain unresolved because their bare namespace cannot be attributed
exactly without compiler expansion.

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

Name matching alone makes no provenance claim. Set `binding = "exact"` to
require the same fail-closed macro-origin resolution used by expansion policy;
an external `source` is accepted only with that explicit exact binding.
`binding = "conservative"` can cover an unresolved exact spelling but cannot
claim external provenance. `zrail explain` identifies the item-macro entry that
actually authorizes each explained file.

External exact binding also requires the dependency's Rust crate root to be
known from Cargo or a matching `dependencies.crate_root` attestation.

```toml
binding = "exact"

[source.rust.item_macros.source]
kind = "registry"
requirement = "0.5"
```

## Cargo

Contract parsing is strict. Unknown keys, stale policy, unresolved source
boundaries, missing evidence, and unreviewed lock changes fail closed.

Cargo package and source analysis follows the active root workspace. Declared
members and in-repository path dependencies are included; unrelated or excluded
nested workspaces remain separate repository boundaries. An active path
dependency that crosses into a nested workspace fails explicitly because zrail
does not guess across multiple workspace inheritance roots.

Repository-controlled Cargo source overrides and registry mappings are rejected
until zrail can attest their effective resolution. Root `.cargo/config` and
`.cargo/config.toml` are rejected because Cargo can use them to alter dependency
resolution and qualification execution.

## Commands

| Command | Purpose |
| --- | --- |
| `zrail init` | Write explicit policy and its initial lock |
| `zrail baseline` | Add reviewed tightening ratchets to an existing contract |
| `zrail check` | Check repository architecture without modifying files |
| `zrail doctor` | Diagnose setup and compatibility problems |
| `zrail explain` | Explain the policy and findings for one path |
| `zrail diff` | Classify architecture changes between trusted states |
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

No release is visible until every target and clean-runtime check succeeds. The
publisher requires the exact archive set, verifies `SHA256SUMS`, creates GitHub
build-provenance attestations, and publishes a draft only after extracting the
matching reviewed section from `CHANGELOG.md`. Release actions are pinned to
full commit identities; the workflow does not accept proposal or manual source
inputs.

`QUAL-02` defines the protected-deployment requirement: proposed checker changes
must not authorize violations or grants in the same pull request. The required
result must come from a ruleset workflow or App outside the proposal's write
domain; the included repository workflow alone does not provide merge authority.

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
