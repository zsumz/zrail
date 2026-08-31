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

### Prebuilt binary

Release archives contain the `zrail` executable, license, and README. Select the
target matching the machine that will run zrail:

| Platform | Target | Archive |
| --- | --- | --- |
| Linux x86-64, glibc | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux ARM64, glibc | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| Linux x86-64, musl | `x86_64-unknown-linux-musl` | `.tar.gz` |
| Linux ARM64, musl | `aarch64-unknown-linux-musl` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

For Linux x86-64 with glibc, download and verify one exact release like this
(replace `0.0.3-rc.6` with the reviewed version):

```sh
ZRAIL_VERSION=0.0.3-rc.6
ZRAIL_TARGET=x86_64-unknown-linux-gnu
ZRAIL_ARCHIVE="zrail-${ZRAIL_VERSION}-${ZRAIL_TARGET}.tar.gz"
ZRAIL_RELEASE="https://github.com/zsumz/zrail/releases/download/v${ZRAIL_VERSION}"
curl -fLO "${ZRAIL_RELEASE}/${ZRAIL_ARCHIVE}"
curl -fLO "${ZRAIL_RELEASE}/SHA256SUMS"
grep "  ${ZRAIL_ARCHIVE}$" SHA256SUMS | sha256sum --check
tar -xzf "${ZRAIL_ARCHIVE}"
./zrail --version
```

Use `LC_ALL=C shasum -a 256 --check` in place of `sha256sum --check` on macOS.
On Windows, compare `Get-FileHash -Algorithm SHA256 <archive>` with the
archive's `SHA256SUMS` entry before expanding the zip. GitHub-hosted provenance
can be verified with:

```sh
gh attestation verify "${ZRAIL_ARCHIVE}" --repo zsumz/zrail
```

CI should use the same checked archive instead of compiling zrail on every run:

```yaml
- name: Download verified zrail binary
  env:
    ZRAIL_VERSION: 0.0.3-rc.6
    ZRAIL_TARGET: x86_64-unknown-linux-gnu
  run: |
    archive="zrail-${ZRAIL_VERSION}-${ZRAIL_TARGET}.tar.gz"
    release="https://github.com/zsumz/zrail/releases/download/v${ZRAIL_VERSION}"
    curl -fLO "${release}/${archive}"
    curl -fLO "${release}/SHA256SUMS"
    grep "  ${archive}$" SHA256SUMS | sha256sum --check
    tar -xzf "${archive}"
    mkdir -p "$HOME/.local/bin"
    install -m 0755 zrail "$HOME/.local/bin/zrail"
- run: zrail check
```

### Cargo fallback

If no prebuilt target fits, install from the locked registry source:

```sh
cargo install zrail --registry crates-io --version 0.0.3-rc.6 --locked
```

Protected tags package, verify, checksum, attest, and publish `zrail-core`,
`zrail-rust`, and `zrail` from the same reviewed checkout. The GitHub release
remains a draft until those exact registry archives are downloadable and match
the attested local bytes.

Building from source requires Rust 1.96 or newer.

## Start

```sh
zrail init --preset rust
zrail baseline --dry-run
zrail baseline --accept-grants
zrail check
zrail explain --path src/lib.rs
```

The `rust` preset fits conventional inline and integration tests. Run
`zrail init` for the zsumz preset: sibling tests, reviewed macro expansion, and
300-line source targets. Plain `init` creates only `zrail.toml` so its authority
can be reviewed before the first baseline and lock. Add `--baseline` for an
atomic contract-and-lock initialization when that review is already complete.

Use repeatable `--exclude PATTERN` options or `--exclude-from FILE` to keep
non-authoritative fixtures out of discovery. The normalized selection is
written exactly into the contract, and it cannot hide an active Cargo target.

If you already maintain `zrail.toml`, preview measurable legacy debt before
granting its exact tightening ratchets:

```sh
zrail check
zrail baseline --dry-run
zrail baseline --accept-grants
zrail check
```

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

Optional `source.rust.feature_worlds` make workspace-wide Cargo feature closure,
`cfg(feature)`, and target `required-features` exact only inside a conservative
subset: every package structurally reachable through a target, build,
development, or proc-macro host context must have no active features. Featureful
split contexts are rejected even when Cargo would unify or converge them;
non-feature target cfgs remain conservative.

`zrail coverage --format json` exports a canonical census of every enabled rail
plus the complete governed owner, dependency, source-syntax, type-policy,
feature-world, and test-mirror surfaces with source spans and analysis quality
for mechanical parity audits.

`zrail mirrors plan --format json` turns large explicit mirror sets into a
digest-bound, execution-grouped plan for a separately trusted test runner;
`zrail mirrors receipts --plan PATH --results PATH --format json` validates a
strict plan-bound result set and renders every schema-2 receipt in one bundle;
`zrail mirrors verify --plan PATH` rejects stale plans and invalid receipts
without executing repository code itself.

Once installed, analysis runs locally and requires no account, daemon, source
upload, API key, LLM, or network access.

## Checks

```text
workspace membership, dependency identity, and layers
runtime and compile-time effects, owners, and source boundaries
facades, test placement, source hygiene, and tightening ratchets
generated source, macro authority, exact test mirrors, and qualification evidence
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

Contract schema and lock-semantics changes use separate review paths:

```sh
zrail migrate-config
zrail migrate-config --write
zrail fmt --check
zrail migrate-lock --base HEAD --output zrail-migration.json
zrail update --accept-migration sha256:<reviewed-report-digest>
```

When the current engine cannot analyze the prior-epoch base, repair the source
on a committed descendant and review a two-revision bridge:

```sh
zrail migrate-lock --base <old-good> --target HEAD --output zrail-migration.json
zrail update --base <old-good> --accept-migration sha256:<reviewed-report-digest> \
  --migration-report zrail-migration.json
```

The bridge binds both commits, both contract and lock identities, the base
analysis failure, and every changed repository file. The target must retain the
exact prior lock. Migration acceptance verifies the named report as the sole
nonignored untracked review artifact, requires tracked worktree bytes and modes
to reproduce the committed target, and never accepts contract grants in the
current worktree.

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
