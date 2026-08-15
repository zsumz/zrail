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

Create a Rust contract and its resolved lock:

```sh
zrail init
```

Check it, inspect one path, or review an architecture change:

```sh
zrail check
zrail explain --path crates/zrail-core/src/lib.rs
zrail diff --base HEAD --deny-grants
```

After an intentional dependency, provenance, gate, or ratchet change:

```sh
zrail update
```

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
- declarative facades, module contracts, and separate tests;
- source hygiene, file-size ceilings, and tightening ratchets; and
- invariants connected to exact tests and content-addressed qualification gates.

Contract parsing is strict. Unknown keys, stale policy, unresolved source
boundaries, missing evidence, and unreviewed lock changes fail closed.

`zrail diff` classifies changes as grants, revocations, debt, cleanup, neutral,
or unknown. `--deny-grants` rejects added power, new debt, and unsafe comparisons.

## CLI

```text
zrail init [ROOT]
zrail check [--root ROOT] [--format human|json]
zrail update [--root ROOT] [--format human|json] [--accept-grants]
zrail doctor [--root ROOT] [--format human|json]
zrail explain --path PATH [--root ROOT] [--format human|json]
zrail diff --base REVISION [--root ROOT] [--deny-grants]
zrail diff --before ROOT --after ROOT [--deny-grants]
```

## Qualification

`QUAL-01` requires the reviewed local gate to remain connected to exact test
evidence. The lock hashes the gate bytes, so a gate change cannot pass silently.

Run the complete offline repository gate with:

```sh
scripts/check
```

It checks structure, formatting, Clippy, tests, rustdoc, zrail itself, package
archives, and a clean diff.

## Status

Version 0.0.1 is the initial release. It supports Rust and Cargo
repositories. zrail analyzes declared source and verified snapshots; it does not
execute repository code, Cargo, build scripts, gates, or generated programs.

## License

Apache-2.0. See [LICENSE](LICENSE).
