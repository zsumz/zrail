# Whole-lock package-name precondition

`KD-PROVENANCE-LOCK-PACKAGE-NAMES` covers the implicit all-entry indexing
precondition in frozen `protocol_provenance.rs:150`: the original whole-array
filter evaluates `package["name"].as_str() == Some(name)` before collecting all
matches. The exact four-line selection is isolated for inspection; the full
original caller/helper remains unchanged and executes separately.

Missing keys and non-table entries panic even if unrelated to the requested
name, including entries after its match. Present non-string names are ignored
by the original filter. Empty and whitespace-only names also do not match the
five required names. The existing native loader rejects all those malformed
names, including unrelated nodes: this is a documented stronger native
constraint, not exact acceptance parity and not new authority.

For accepted names, both selectors are literal and case-sensitive. No case,
dash/underscore, whitespace or Unicode normalization is applied. The native
lock loader checks nonempty strings, not the entire Cargo package-name grammar;
this proof must not invent additional name restrictions.

## Evidence scope

The 32-case synthetic matrix holds the earlier array and lock-version conditions
valid. It compares complete original counts with the existing native whole-lock
policy's observed counts, not its full identity acceptance. Synthetic graphs
have no active workspace packages. Missing/non-table, non-string/blank and later
version/duplicate-node failures remain separate stages.

Sixteen frozen-input mutations invoke the complete original guard and the native
loader with all five existing identity policies and actual workspace packages.
Errors stop at the first failed check; only successful native cases reach all
five policy observations. The report's five original helper invocations describe
a successful complete original run, not an early-failing mutation. Eleven native
rejections include three missing-name cases (before, within and after the full
83-node inventory) and eight stricter non-string/blank cases. Five accepted cases
preserve the exact required identities. Only a fresh test-owned copy's lockfile
changes; all twelve input hashes and the complete mutated lock source are recorded.

The trusted runner requires five exact tests, including explicit frozen
qualification. Two clean runs at signed producer
`9ee1bced403ea609d931718357ab847ed896c400`, tree
`916596d7e24de0577224e3ebf084b0bd06e54ac9`, produced byte-identical 419,017-byte
reports with SHA-256
`372a5e4baf3b4e22f3dd6a10e666c1c5faf0f484245bae1c1059c467d4ec01d9`.
The [index](../evidence/lock-names-index.json) binds all 24 raw reports/logs,
23 exact producer inputs, the 649-test executable inventory and compiler.
Seven validator tests reject 133 mutations; two runner tests reject 35 malformed
execution outcomes. Snapshot verification passes 22 extraction registries and
213 original policy instances before and after each run.

The ledger verifies only this one precondition with `full_snapshot_verified`
false. Native name errors are graph-loader errors, not stable finding IDs;
the exact messages are bound without inventing diagnostic codes. No root
contract, policy fragment, lock, downstream guard, grant or release state changes.
Version-prefix authority and full qualification remain open.
