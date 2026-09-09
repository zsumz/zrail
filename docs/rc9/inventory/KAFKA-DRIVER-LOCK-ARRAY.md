# Lock package-array precondition

`KD-PROVENANCE-LOCK-PACKAGE-ARRAY` binds the exact three-line array precondition
at `protocol_provenance.rs:145` in frozen kafka-driver
`a45be8071e6a663cd4ae3142cc5304ece9fdf45e`. The original caller, complete
`assert_locked` helper, read/parse functions and registry remain unchanged in
the existing lock-package harness. A separately bound exact excerpt isolates
the array check from version-prefix and package-count failures.

## Existing native enforcement

The native Cargo.lock parser already requires the root `package` field to be
an array. Missing and non-array values produce `Cargo.lock requires [[package]]
entries`; the original either fails indexing a missing key or emits its frozen
array error. No new engine rule or policy is needed. The five existing
`dependency:lock-package:kd-lock-*` rules depend on this mandatory loader.

An empty array passes this type stage. A synthetic graph with no active workspace
packages can therefore be empty, but all five existing required identity/count
rules then fail with `DEP-LOCK-001`. The original full helper rejects those same
five missing packages through its separate exact-count assertion. An actual
consumer graph additionally requires its active workspace nodes; the synthetic
empty graph is not a qualified consumer lock.

| Input shape | Original array stage | Native result |
| --- | --- | --- |
| Missing or non-array `package` | Reject | Reject at the array check |
| Empty array, supported lock version | Accept | Empty graph; required inventories reject later |
| Valid registry-node arrays | Accept | Complete graph |
| Array containing invalid package entries | Accept | Existing later node validation rejects |
| Array with missing or unsupported lock version | Accept | Existing earlier version validation rejects |

The 24-case matrix records these stages independently. Its parser inputs are
well-formed TOML; it does not claim malformed-document or I/O error parity.
Synthetic cases deliberately have no active workspace packages. Frozen
qualification instead derives all five real workspace packages from twelve
bound inputs, executes every original helper invocation, and verifies the
five existing rules over all 83 lock nodes. No source-analysis certificate is
produced, and package-name and exact-version preconditions remain separate work.

## Evidence boundary

The trusted runner requires five exact tests, including explicit frozen
qualification. Verification remains open until two clean signed-producer runs,
raw execution logs and artifact/linkage rejection checks are bound. No root
contract, lock, policy fragment, downstream guard, grant or release state changes.
