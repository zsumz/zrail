# Exact-version prefix translation

The five `KD-PROVENANCE-*-VERSION-PREFIX` instances share the exact original
three-line `str::strip_prefix('=')` operation at `protocol_provenance.rs:142`.
It strips one leading equals sign without trimming, parsing or normalization.
The complete original helper then compares the remaining text with the lock
version as a string. The prefix operation alone accepts even an empty or
malformed suffix; a paired malformed lock can satisfy that original helper.

## Native representation and trusted conversion

Native `kind = "exact"` policies store the version without a leading equals
sign. The contract validator accepts bounded literal strings, not a semver
requirement grammar. Cargo.lock parsing separately requires valid observed
semantic versions. A range-looking expected string is never interpreted as a
range: it either cannot equal an observed valid Cargo version or yields an exact
identity mismatch. Build metadata remains part of string identity.

The trusted `rc9-provenance-policies` translator now requires one prefix assertion
per package, the exact source/registry linkage and selector, unchanged prefix
semantics, and an exact suffix match with the already reviewed native identity.
Its existing CLI continues to verify frozen source and registry hashes and
registry scalar equality before generating policy. It does not accept a new
registry version or turn a bare legacy version into authority. The generated
five-rule fragment remains byte-identical. General registry parsing/schema
translation and full repository qualification remain separate work.

## Qualification scope

Twenty-four forms across all five packages produce 120 synthetic pairs. These
record original prefix and complete-helper outcomes, native contract validation,
native Cargo parsing and exact observations separately. The original helper is
given a matching literal lock version, exposing malformed suffixes it accepts.
The bare native literal is a deliberate representation boundary: native can
accept it, but the converter still rejects the unprefixed legacy input.
Valid alternate fixture identities are not accepted grants.

Twenty further comparisons use the full frozen 83-node lock and all five actual
workspace packages. Caret, wildcard, changed version and changed build metadata
all fail exact matching with `DEP-LOCK-001`, despite syntactically valid native
policy literals. The original helper fails its later equality check, not its
prefix check. The unmodified full original caller and all five native policies
also run over twelve frozen inputs. Synthetic pairs have no active workspace
packages and are not full consumer qualification.

The runner additionally executes the real converter CLI and verifies identical
fragment bytes, plus 120 conversion cases and 95 authority-linkage rejections.
Evidence linkage remains open until two clean signed-producer runs, all raw
logs, and artifact/linkage validation are bound. No runtime engine rule, root
contract, reviewed lock, existing policy, downstream guard or release state changes.

The initial focused run failed because the harness serialized a nested assertion
as a top-level table. The corrected harness constructs the complete typed TOML
document and binds its actual bytes. The failed `lock-prefix-focused.log` remains
diagnostic-only under `target/rc9-completion-20260907/`; no native validation rule
was changed to make it pass.
