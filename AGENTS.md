# Agent instructions

Read `README.md` and the owning module contract before editing code.

## Loop

1. Keep each change inside one coherent ticket-sized scope.
2. Keep every Rust source file at or below 300 lines.
3. Keep `lib.rs`, `main.rs`, and `mod.rs` declarative.
4. Put tests in sibling `*_test.rs` files or integration-test directories.
5. Run `scripts/check` before finishing.
6. When `zrail.toml` or `zrail.lock` changes, run `zrail diff` and report every
   grant, debt increase, or unknown change.

Do not weaken a rail merely to make an implementation pass.

Never run `zrail update --accept-grants`, weaken `zrail.toml`, or replace
`zrail.lock` without explicit human authorization. Report the semantic diff
instead.

`zrail review --allow-grants` is for explicit human review. It must never appear
in automated or proposal-controlled merge checks; only separately protected,
human-dispatched authority may use it.
