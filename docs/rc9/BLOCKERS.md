# Open reviewed assertion blockers

Release verdict: **blocked**. This list names each reviewed but unverified
assertion instance. It does not close the additional discovery blockers
`RC9-INVENTORY-KD`, `RC9-INVENTORY-KF`, and `RC9-INVENTORY-RF`.

All unreviewed candidate IDs are enumerated in `census.json.gz`. Non-Rust
files and opaque or invalid syntax remain explicit review work.
`RF-PRIVATE-NAME-PATTERNS` is closed by the user's approved retirement in
[RC9-DECISION-RF-PRIVATE-NAMES](DECISIONS.md).

| Assertion ID | Cause |
| --- | --- |
| `KD-CI-ENFORCEMENT` | The mission explicitly requires replacing this prohibition with an enforced zrail lane. The new lane and its required policy/receipt evidence are not yet qualified. |
| `KD-FACADE-LIVE` | Predicate fixtures pass; full downstream selection and complete declarative policy qualification remain open. |
| `KD-LOCK-BAN-async-std` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-BAN-async-trait` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-BAN-smol` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-BAN-tokio` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-BAN-tokio-rustls` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-BAN-tokio-util` | Whole-lock scope must be preserved; reachable transitive dependency bans alone are insufficient. |
| `KD-LOCK-bornera-CHECKSUM` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-COUNT` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-SOURCE` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-VERSION` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-core-CHECKSUM` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-core-COUNT` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-core-SOURCE` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-core-VERSION` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-rustls-CHECKSUM` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-rustls-COUNT` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-rustls-SOURCE` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-bornera-rustls-VERSION` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-CHECKSUM` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-COUNT` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-SOURCE` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-VERSION` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-core-CHECKSUM` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-core-COUNT` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-core-SOURCE` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-LOCK-kafka-wire-core-VERSION` | Requires whole-lock selected cardinality and exact provenance parity; no replacement bundle has been qualified. |
| `KD-TRANSPORT-ASSOCIATED` | Exact inventory predicates, authored alias restrictions, compilation-world rules, and positive/negative full selection qualification remain open. |
| `KD-TRANSPORT-IMPLS` | Exact inventory predicates, authored alias restrictions, compilation-world rules, and positive/negative full selection qualification remain open. |
| `KD-TRANSPORT-METHODS` | Exact inventory predicates, authored alias restrictions, compilation-world rules, and positive/negative full selection qualification remain open. |
| `KD-TRANSPORT-OWNERS` | Exact inventory predicates, authored alias restrictions, compilation-world rules, and positive/negative full selection qualification remain open. |
| `KD-TRANSPORT-RENAMES` | Exact inventory predicates, authored alias restrictions, compilation-world rules, and positive/negative full selection qualification remain open. |
| `KF-BUDGET-ALLOW-PATH` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-ALLOW-PRESENCE` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-ALLOW-UNIQUE` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-BASELINE-PATH` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-BASELINE-PRESENCE` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-BASELINE-UNIQUE` | Predicate implementation exists; exact frozen positive/negative mapping remains to qualify. |
| `KF-BUDGET-DECOY-FIXTURE` | Required detector input must be imported and bound; this assertion is not classified as system behavioral evidence. |
| `KF-BUDGET-NESTED-TESTS` | Future nested path classification passes; the exact nested-decoy-manifest counterexample still needs import and complete Cargo/source qualification. |
| `KF-FACADE-LIVE` | Predicate fixtures pass; full downstream selection and complete declarative policy qualification remain open. |
| `KF-FETCH-ORDER-ACTIVATION` | Bounded code-order predicate is absent; its stronger code-only semantics need explicit difference evidence. An owner allowlist is insufficient. |
| `KF-FETCH-ORDER-CONTROL` | Bounded code-order predicate is absent; its stronger code-only semantics need explicit difference evidence. An owner allowlist is insufficient. |
| `KF-FETCH-ORDER-PREPARE` | Bounded code-order predicate is absent; its stronger code-only semantics need explicit difference evidence. An owner allowlist is insufficient. |
| `KF-FETCH-PRESENCE-01` | Required literal/code predicate translation and omission fixtures remain open. |
| `KF-FETCH-PRESENCE-02` | Required literal/code predicate translation and omission fixtures remain open. |
| `KF-FETCH-PRESENCE-03` | Required literal/code predicate translation and omission fixtures remain open. |
| `KF-SCAN-MINIMUM` | Repository-file assertions and exact source-selection fixtures are required; passing the frozen scan does not supply a replacement guard. |
| `KF-SCAN-ROOTS` | Native repository.roots already checks directory presence; the frozen positive/negative source-selection qualification still needs intended-diagnostic linkage. |
| `RF-FACADE-LIVE` | Rafter permits data declarations; wiring modes would change policy. Exact raw-line or reviewed structural normalization remains open. |
| `RF-PROCESS-ALLOWLIST` | Exact contextual inventory and non-aliasing predicates are not implemented or differentially verified. |
| `RF-PROCESS-CRATE-ALIASES` | Exact contextual inventory and non-aliasing predicates are not implemented or differentially verified. |
| `RF-PROCESS-EXACT` | Exact contextual inventory and non-aliasing predicates are not implemented or differentially verified. |
| `RF-PROCESS-MACROS` | Exact contextual inventory and non-aliasing predicates are not implemented or differentially verified. |
| `RF-PROCESS-SPELLING` | Exact contextual inventory and non-aliasing predicates are not implemented or differentially verified. |
| `RF-TEST-FACADE-LIVE` | Rafter permits data declarations; wiring modes would change policy. Exact raw-line or reviewed structural normalization remains open. |
| `RF-WIRE-ALIAS-ENTRY` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-ALIAS-MEMBERSHIP` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-ALIAS-MSG` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-REGISTRY-LogEntryTag` | Required source/text predicates are not translated. |
| `RF-WIRE-REGISTRY-MembershipTag` | Required source/text predicates are not translated. |
| `RF-WIRE-REGISTRY-MessageTag` | Required source/text predicates are not translated. |
| `RF-WIRE-RESERVATION` | Required documentation literal predicate is not translated. |
| `RF-WIRE-U8-SHAPES` | Call-argument shapes and exact exception parity are not implemented; ownership does not subsume this requirement. |
