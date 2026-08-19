<p align="center">
  <img src="https://raw.githubusercontent.com/zsumz/zrail/main/zrail-logo.svg" alt="zrail" width="720">
</p>

<p align="center"><strong>Executable guardrails for human- and agent-written code.</strong></p>

<p align="center">
  Deterministic architecture checks, path-specific guidance, and reviewable
  diffs for Rust and Cargo repositories.
</p>

<p align="center">
  <a href="#install">Install</a>
  <span> · </span>
  <a href="#start">Start</a>
  <span> · </span>
  <a href="#model">Model</a>
  <span> · </span>
  <a href="#review">Review</a>
  <span> · </span>
  <a href="#crates">Crates</a>
  <span> · </span>
  <a href="https://github.com/zsumz/zrail/blob/main/docs/GUIDE.md">Guide</a>
</p>

<br />

## Install

```sh
cargo install zrail --locked
```

Rust 1.96 or newer is required.

## Start

```sh
zrail init --preset rust
zrail check
zrail explain --path src/lib.rs
```

The `rust` preset fits conventional inline and integration tests. Run
`zrail init` for the zsumz preset: sibling tests, reviewed macro expansion, and
300-line source targets. Add `--baseline` when adopting existing code.

After committing the initial `zrail.toml` and `zrail.lock`, review later changes
with:

```sh
zrail diff --base HEAD --deny-grants
```

## Model

```text
zrail.toml   human-authored architecture
zrail.lock   resolved, content-bound architecture state
Cargo.lock   Cargo's selected versions and checksums
```

`zrail check` is read-only. `zrail diff` separates grants, revocations, debt,
cleanup, neutral changes, and unknown comparisons.

Once installed, analysis runs locally and requires no account, daemon, source
upload, API key, LLM, or network access.

## Checks

```text
workspace membership, dependency identity, and layers
runtime and compile-time effects, owners, and source boundaries
facades, test placement, source hygiene, and tightening ratchets
generated source, macro authority, and qualification evidence
```

Unknown configuration, stale policy, unresolved source boundaries, missing
evidence, and unreviewed lock changes fail closed.

## Review

```sh
zrail review --base HEAD --authority-root . --root proposal
```

A binary built from protected source can analyze a proposal as data and verify
its checked-in lock against independently observed architecture. Violations,
grants, new debt, unknown comparisons, and stale or forged locks are rejected by
default.

`--accept-grants` and `--allow-grants` require explicit human review. They must
not appear in automated or proposal-controlled merge checks.

## Crates

| Crate | Purpose |
| --- | --- |
| `zrail` | CLI for checks, explanations, diffs, and review |
| `zrail-core` | Language-neutral contracts, locks, diagnostics, and diffs |
| `zrail-rust` | Rust and Cargo analysis |

## Guide

The [guide and trust model](https://github.com/zsumz/zrail/blob/main/docs/GUIDE.md)
covers adoption, lock updates, macros, generated source, protected review, and
qualification.

## Development

```sh
cargo install --path crates/zrail-cli --locked
```

## Qualification

```sh
scripts/check
```

This is the complete offline gate for structure, formatting, Clippy, tests,
rustdoc, zrail itself, package archives, and repository cleanliness.

## Scope

zrail supports Rust and Cargo repositories. It analyzes declared source and
reviewed snapshots; it does not execute repository code, Cargo, build scripts,
gates, or generated programs.

## License

Apache-2.0. See [LICENSE](LICENSE).
