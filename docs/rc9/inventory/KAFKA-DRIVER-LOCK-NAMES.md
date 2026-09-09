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

Sixteen frozen-input mutations execute the complete original guard and all five
native identity policies with all five actual workspace packages. Eleven native
rejections include three missing-name cases (before, within and after the full
83-node inventory) and eight stricter non-string/blank cases. Five accepted cases
preserve the exact required identities. Only a fresh test-owned copy's lockfile
changes; all twelve input hashes and the complete mutated lock source are recorded.

The trusted runner requires five exact tests, including explicit frozen
qualification. Evidence linkage remains open until two clean signed-producer
runs and their complete raw logs are bound and adversarial validation passes.
No root contract, policy fragment, lock, downstream guard, grant or release
state changes. Version-prefix authority and full qualification remain open.
