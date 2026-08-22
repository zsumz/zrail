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
(replace `0.0.1` with the reviewed version):

```sh
ZRAIL_VERSION=0.0.1
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
    ZRAIL_VERSION: 0.0.1
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
cargo install zrail --locked
```

Building from source requires Rust 1.96 or newer.

## Start

```sh
zrail init --preset rust
zrail check
zrail explain --path src/lib.rs
```

The `rust` preset fits conventional inline and integration tests. Run
`zrail init` for the zsumz preset: sibling tests, reviewed macro expansion, and
300-line source targets. Add `--baseline` when adopting existing code.

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
