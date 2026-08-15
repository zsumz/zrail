<p align="center">
  <img src="./zrail-logo.svg" alt="zrail" width="720">
</p>

<p align="center"><strong>Executable guardrails for human- and agent-written code.</strong></p>

zrail is a deterministic architecture checker for repositories. It turns
repository rules into local checks, path-specific guidance, and semantic diffs.
It requires no account, daemon, source upload, network request, API key, or LLM.

## Install

From this checkout:

```sh
cargo install --path crates/zrail-cli --locked
```


## Use

Create a zsumz-style contract and its resolved lock:

```sh
zrail init
```

The default `zsumz` preset uses sibling test files and 300-line source targets.
For conventional Rust organization, including inline unit tests and Cargo
integration tests, use:

```sh
zrail init --preset rust
```

The `rust` preset does not invent a universal source-file length limit. Both
presets write an explicit `zrail.toml`; the preset is not hidden policy.

Add `--baseline` to either preset when adopting an existing repository:

```sh
zrail init --baseline
zrail init --preset rust --baseline
```

Baseline records supported violations of the selected preset as exact,
tightening ratchets. New debt fails, cleanup makes the lock stale so it must
tighten, and unsupported violations abort initialization without partial files.

Check it, inspect one path, or review an architecture change:

```sh
zrail check
zrail explain --path crates/zrail-core/src/lib.rs
zrail diff --base HEAD --deny-grants
```

Protected automation can independently derive a proposal's observed state with
a binary and base revision outside the proposal's control:

```sh
zrail review --base HEAD --authority-root . --root proposal --deny-grants
```

`review` analyzes the proposed Cargo and Rust source as data, verifies its
checked-in lock against an in-memory candidate, and rejects source violations,
grants, new debt, unknown comparisons, and stale or forged locks.

The included authority workflow runs on `pull_request_target`, builds only the
protected base checkout, and places the proposal beneath that trusted root only
after the authority binary exists. It executes no proposal actions or scripts.
Protect the branch and require `zrail/protected-source-review`; the workflow
publishes that status on the exact proposal commit. The ordinary CI workflow
also handles merge-group events. A merge queue that requires protected review
needs an organization ruleset workflow or external authority service capable of
reviewing the merge-group commit.

After an intentional dependency, provenance, gate, or ratchet change:

```sh
zrail update
```

`zrail update` uses committed `HEAD` architecture as its authority. Choose a
different independently reviewed revision with `--base REVISION`.

Policy changes and `zrail update --accept-grants` require explicit human
authorization. Agents should report `zrail diff` instead of accepting power.

`zrail.toml` contains human-authored architecture. `zrail.lock` contains exact
resolved state, reviewed gate bytes, generated provenance, and ratchets.
`zrail check` never modifies either file.

## Checks

The Rust adapter enforces:

- exact Cargo workspace membership and dependency edges;
- package layers and allowed dependency direction;
- effect boundaries, capability owners, and direct-call owners;
- complete Rust source traversal, including reviewed generated and `OUT_DIR`
  snapshots;
- declarative facades, module contracts, and configurable test placement;
- source hygiene, optional file-size ceilings, and tightening ratchets; and
- invariants connected to exact tests and content-addressed qualification gates.

Contract parsing is strict. Unknown keys, stale policy, unresolved source
boundaries, missing evidence, and unreviewed lock changes fail closed.

`zrail diff` classifies changes as grants, revocations, debt, cleanup, neutral,
or unknown. `--deny-grants` rejects added power, new debt, and unsafe comparisons.

## CLI

```text
zrail init [ROOT] [--preset zsumz|rust] [--baseline]
zrail check [--root ROOT] [--format human|json]
zrail update [--base REVISION] [--root ROOT] [--format human|json] [--accept-grants]
zrail doctor [--root ROOT] [--format human|json]
zrail explain --path PATH [--root ROOT] [--format human|json]
zrail review [--base REVISION] [--authority-root ROOT] --root PROPOSAL [--deny-grants]
zrail diff --base REVISION [--root ROOT] [--deny-grants]
zrail diff --before ROOT --after ROOT [--deny-grants]
```

## Qualification

`QUAL-01` requires the reviewed local gate to remain connected to exact test
evidence. The lock hashes the gate bytes, so a gate change cannot pass silently.

`QUAL-02` requires pull requests to pass independent source analysis by a zrail
binary built from the protected base commit. It verifies observed source and the
proposed lock, so proposed checker changes cannot authorize violations or grants
in the same pull request.

Run the complete offline repository gate with:

```sh
scripts/check
```

It checks structure, formatting, Clippy, tests, rustdoc, zrail itself, package
archives, whitespace, and that Git status is unchanged by the gate.

## Status

Version 0.0.1 is the initial release. It supports Rust and Cargo
repositories. zrail analyzes declared source and verified snapshots; it does not
execute repository code, Cargo, build scripts, gates, or generated programs.

## License

Apache-2.0. See [LICENSE](LICENSE).
