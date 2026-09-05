This crate reads Cargo manifests, Rust source, [`zrail.toml`](https://github.com/zsumz/zrail)
contracts, and optional zrail locks as data. It does not invoke Cargo, build
scripts, procedural macros, qualification gates, or repository programs, and
its public operations do not write to the analyzed repository.

[`check_repository`] is the primary integration point. It returns both a
diagnostic report and, when analysis is complete, an independently observed
candidate lock. [`build_lock`] exposes it for explicitly authorized lock updates.
Relative configuration, lock, and explained paths are beneath the repository root.

The baseline discovery types are public initialization support for the `zrail`
CLI. They describe conservative source roots and exact debt ratchets; they do
not modify a contract or lock themselves.
