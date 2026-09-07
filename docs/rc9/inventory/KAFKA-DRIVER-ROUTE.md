# Mixed route-test assertion review

Frozen source: `kafkars/kafka-driver@a45be8071e6a663cd4ae3142cc5304ece9fdf45e`,
`src/reactor/direct_plaintext/cluster_runtime/route_state_test.rs`, SHA-256
`f9d1bcc036579e23d55b7d9f23d4d2aa95929f1fbe514b6a1397799107675be3`.

This file was missing from the assertion ledger. All 28 of its census candidates
are now linked to 31 assertion instances. Its two tests have different purposes;
classifying the whole file as behavioral would lose architecture policy.

| Assertion IDs | Original property | Disposition |
| --- | --- | --- |
| `KD-ROUTE-FORBID-{ConnectionSet,Poller,SingleBroker,BrokerSet}` | Each of ten literal source inputs omits one case-sensitive raw token. Comments, string literals, aliases and inactive cfg text participate. | Declarative translation |
| `KD-ROUTE-OWNER-FIELD` | The separately included facade has exactly one nonoverlapping `connections: DirectSetOwner<T>` text occurrence. This is raw text, including comments; it does not prove field identity. | Declarative translation |
| `KD-ROUTE-INPUT-*` (11 instances) | Every literal `include_str!` input must exist and decode as UTF-8, including when a forbidden token is absent. | Declarative translation |
| `KD-ROUTE-RUNTIME-*` (15 instances) | Actual directory installation, route submission and DNS completion preserve owners, open only demanded lanes, retain queued requests and consume pending work. Four instances are required successful setup/submission outcomes; eleven compare runtime results. | Behavioral evidence retained |

The [policy fragment](../policies/kafka-driver.route-sources.fragment.toml) uses
six existing stock repository-file rules: one exact physical input set, four
literal prohibitions and one exact literal count. It adds no analyzer capability,
local evaluator gate or semantic type claim. Compile-input dependency links
prevent marking a text assertion verified while its required inputs remain
unverified. An unrelated file cannot satisfy the selected facade's count.

The testkit copies all eleven inputs and the original mixed test, binding their
bytes through `route-sources-origins.json`. The trusted minimal harness executes
the unchanged forbidden-name loop and exact-count assertion with injected text.
Focused tests mutate every selected route file in comments, strings and renamed
imports; preserve case-sensitive acceptance; test exact counts and wrong-file
decoys; and require missing/invalid UTF-8 inputs to fail with the intended native
policy or completeness diagnostic.

The sixteen source/input translations are verified at
`f552b5387b8d7941e8b52facdfe8e7e8829889b4`. Two byte-identical reports cover 189
cases each (43 accepted, 146 rejected) and 23 original compilations: one clean
source-only test and missing/invalid-UTF-8 failures for each of eleven inputs.
The whole original source-only function is extracted byte-for-byte and checked
for AST equality; its single runnable, nonignored test must pass. Missing-input
diagnostics identify the original include; invalid-UTF-8 diagnostics identify
the selected file's invalid byte. Every fixture binds the eleven selected inputs
and any unrelated-file decoy.

The [evidence index](../evidence/route-source-index.json) preserves both runs and
the initial harness failure. The trusted artifact validator checks all policy
selectors, raw quantities, input hashes, original outcomes and compiler sites;
42 tamper cases fail. These producers and validators stay outside zrail's runtime.

The fifteen retained runtime assertions still have no execution receipt from
this slice. No downstream test or helper was changed or removed. A future cutover
may remove only the source-only function after complete repository qualification;
the directory/routing/DNS/owner-reuse scenario and mixed file remain.
