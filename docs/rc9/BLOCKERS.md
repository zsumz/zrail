# Open reviewed assertion blockers

Release verdict: **blocked**. This list names each reviewed but unverified
assertion instance. It does not close the additional discovery blockers
`RC9-INVENTORY-KD`, `RC9-INVENTORY-KF`, and `RC9-INVENTORY-RF`.
The fallible-call discovery gap is closed by the expanded census; its new
candidates still require review. `RC9-INVENTORY-FRAGMENTS` records seven Rust
`.inc` inputs whose candidate discovery must be regenerated from the bound
`additional-rust-inputs.json` registry.

All unreviewed candidate IDs are enumerated in `census.json.gz`. Non-Rust
files and opaque or invalid syntax remain explicit review work.
`RF-PRIVATE-NAME-PATTERNS` is closed by the user's approved retirement in
[RC9-DECISION-RF-PRIVATE-NAMES](DECISIONS.md).

| Assertion ID | Cause |
| --- | --- |
| `KD-CI-ENFORCEMENT` | The mission explicitly requires replacing this prohibition with an enforced zrail lane. The new lane and its required policy/receipt evidence are not yet qualified. |
| `KD-DEP-DETECTOR-EXACT-NAME` | Import the exact detector fixture and execute its line-name extractor beside native line policies. |
| `KD-DEP-DETECTOR-PRESENT-NAME` | Import the exact detector fixture and execute its line-name extractor beside native line policies. |
| `KD-FACADE-LIVE` | Predicate fixtures pass; full downstream selection and complete declarative policy qualification remain open. |
| `KD-FACADE-PARSE` | Stock wiring-only and test-facade policies already reject non-item fragments through RUST-FACADE-002, and malformed source fails parsing. Bind every frozen facade and the exact original implementation_items helper to positive/negative differential fixtures; preserve rc8 declarative fragment behavior. |
| `KD-METADATA-PARENT` | Record the path-precondition proof and bind the exact translated license paths before closing this helper assertion. |
| `KD-PROVENANCE-LOCK-PACKAGE-ARRAY` | Translate the exact authored provenance predicate and qualify frozen positive/negative inputs; no complete protocol policy bundle has been verified. |
| `KD-PROVENANCE-LOCK-PACKAGE-NAMES` | Account for the implicit all-entry name-index precondition. Existing Cargo parsing rejects non-string names more strictly; qualify this distinction and preserve complete lock scope. |
| `KD-PROVENANCE-bornera-VERSION-PREFIX` | Preserve the exact-version input condition in canonical native package identity policy and qualify the representation change; a ranged or malformed expected version cannot become authority. |
| `KD-PROVENANCE-bornera-core-VERSION-PREFIX` | Preserve the exact-version input condition in canonical native package identity policy and qualify the representation change; a ranged or malformed expected version cannot become authority. |
| `KD-PROVENANCE-bornera-rustls-VERSION-PREFIX` | Preserve the exact-version input condition in canonical native package identity policy and qualify the representation change; a ranged or malformed expected version cannot become authority. |
| `KD-PROVENANCE-kafka-wire-VERSION-PREFIX` | Preserve the exact-version input condition in canonical native package identity policy and qualify the representation change; a ranged or malformed expected version cannot become authority. |
| `KD-PROVENANCE-kafka-wire-core-VERSION-PREFIX` | Preserve the exact-version input condition in canonical native package identity policy and qualify the representation change; a ranged or malformed expected version cannot become authority. |
| `KD-REGISTRY-PARSE` | Complete the typed registry-to-zrail conversion and schema compatibility evidence. Do not retain an unexecuted legacy registry as proof of translated policy; source registry read and type failures need explicit fixture bindings. |
| `KD-REGISTRY-SCHEMA` | Bind the supported source schema to the trusted conversion and resulting strict zrail contract schema; malformed, absent and unsupported versions need original/native comparison. |
| `KD-RETIRED-BACKEND-VARIANT` | Bounded Rust enum-variant predicate and cfg/omission/duplicate fixtures remain open. |
| `KD-RETIRED-CONSTRUCTION-LegacyBackend` | Raw absence predicate exists; the pinned source selection and differential evidence remain to qualify. |
| `KD-RETIRED-CONSTRUCTION-new_legacy` | Raw absence predicate exists; the pinned source selection and differential evidence remain to qualify. |
| `KD-RETIRED-MODULE-broker_set` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-plaintext` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-poller` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-resource` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-tcp` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-timer` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-MODULE-tls` | Required Rust module-declaration absence predicate and adversarial source fixtures remain open. |
| `KD-RETIRED-PARSE-BACKEND` | Bind this existing source parse requirement to the explicit source inventory and prove malformed/absent input failure. |
| `KD-RETIRED-PARSE-MODULES` | Bind this existing source parse requirement to the explicit source inventory and prove malformed/absent input failure. |
| `KD-RETIRED-TREE-broker_set` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-plaintext` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-poller` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-resource` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-tcp` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-timer` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-RETIRED-TREE-tls` | A bounded physical prohibition exists; preserve the exact descendant scope and record stronger explicit read/symlink failure behavior in differential evidence. |
| `KD-TEST-PLACEMENT-DETECTOR` | Execute the exact frozen detector fixture and compare the structural observations, including original order and cardinality; a generic nonzero zrail status is insufficient. |
| `KD-TEST-PLACEMENT-LIVE` | Preserve the top-level selection and intentional raw cfg-token predicate, or obtain a separately approved normalization. Semantic cfg(test) detection alone permits legacy-forbidden cfg(not(test)) and feature="latest" inline modules; nested detection also differs. No policy/diagnostic fixtures yet. |
| `KD-TEST-PLACEMENT-PARSE` | The authored inventory already requires strict file syntax over the same frozen production path selection. Bind the original test_body_kinds helper and its exact detector source to differential parser fixtures before claiming this separate assertion verified. |
| `KD-TRANSPORT-ASSOCIATED` | Native written-expression-path quantities and 22 frozen-collector syntax comparisons are implemented. Bind all frozen selected files, the exact original detector assertion and complete positive/negative count maps to reproducible differential evidence. |
| `KD-TRANSPORT-DETECTOR-ASSOCIATED` | Native written-expression-path quantities and 22 frozen-collector syntax comparisons are implemented. Bind all frozen selected files, the exact original detector assertion and complete positive/negative count maps to reproducible differential evidence. |
| `KD-TRANSPORT-DETECTOR-IMPLS` | Import the exact adversarial source and compare complete native observations with this original detector assertion; reject its architectural violations through the linked live inventory policies. |
| `KD-TRANSPORT-DETECTOR-OWNERS` | Import the exact adversarial source and compare complete native observations with this original detector assertion; reject its architectural violations through the linked live inventory policies. |
| `KD-TRANSPORT-DETECTOR-RENAMES` | Import the exact adversarial source and compare complete native observations with this original detector assertion; reject its architectural violations through the linked live inventory policies. |
| `KD-TRANSPORT-IMPLS` | Preserve the exact authored occurrence context, identity/set/count semantics and full physical selection with native inventories; qualify original detector outputs, aliases, cfg branches, macros and all unexpected/missing members. |
| `KD-TRANSPORT-OWNERS` | Preserve the exact authored occurrence context, identity/set/count semantics and full physical selection with native inventories; qualify original detector outputs, aliases, cfg branches, macros and all unexpected/missing members. |
| `KD-TRANSPORT-RENAMES` | Preserve the exact authored occurrence context, identity/set/count semantics and full physical selection with native inventories; qualify original detector outputs, aliases, cfg branches, macros and all unexpected/missing members. |
| `KD-TRAVERSAL-DIRECTORY` | Qualify complete selected-root traversal and original I/O failure behavior, including missing and empty roots, unreadable directories, entry inspection failure, and symlink differences. Bind the intended completeness diagnostic before claiming parity. |
| `KD-TRAVERSAL-ENTRY` | Qualify complete selected-root traversal and original I/O failure behavior, including missing and empty roots, unreadable directories, entry inspection failure, and symlink differences. Bind the intended completeness diagnostic before claiming parity. |
| `KD-TRAVERSAL-TYPE` | Qualify complete selected-root traversal and original I/O failure behavior, including missing and empty roots, unreadable directories, entry inspection failure, and symlink differences. Bind the intended completeness diagnostic before claiming parity. |
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
| `RF-PROCESS-ALLOWLIST` | Qualify the exact authored Import/all-Path occurrence contexts, module-graph normalization, written spelling, extern-crate and flattened macro-token predicates. Preserve each exact tuple/count and test the original detector; semantic call ownership alone is insufficient. |
| `RF-PROCESS-CRATE-ALIASES` | Qualify the exact authored Import/all-Path occurrence contexts, module-graph normalization, written spelling, extern-crate and flattened macro-token predicates. Preserve each exact tuple/count and test the original detector; semantic call ownership alone is insufficient. |
| `RF-PROCESS-DETECTOR-CRATE-ALIASES` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-DETECTOR-MACROS` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-DETECTOR-PARSE` | Bind the exact original fixture parse precondition to native strict syntax and malformed/expression-only detector fixtures; no qualified replacement policy identity or diagnostic is linked yet. |
| `RF-PROCESS-DETECTOR-ROOT-IMPORT` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-EXACT` | Qualify the exact authored Import/all-Path occurrence contexts, module-graph normalization, written spelling, extern-crate and flattened macro-token predicates. Preserve each exact tuple/count and test the original detector; semantic call ownership alone is insufficient. |
| `RF-PROCESS-MACROS` | Qualify the exact authored Import/all-Path occurrence contexts, module-graph normalization, written spelling, extern-crate and flattened macro-token predicates. Preserve each exact tuple/count and test the original detector; semantic call ownership alone is insufficient. |
| `RF-PROCESS-NORMALIZE-IMPORT-SELF` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-NORMALIZE-SUPER` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-PARSE` | Translate this exact original collector/parser assertion and bind its positive/negative fixture, intended native policy identity and diagnostic. No behavioral retention or implicit retirement applies. |
| `RF-PROCESS-SPELLING` | Qualify the exact authored Import/all-Path occurrence contexts, module-graph normalization, written spelling, extern-crate and flattened macro-token predicates. Preserve each exact tuple/count and test the original detector; semantic call ownership alone is insufficient. |
| `RF-TEST-FACADE-LIVE` | Rafter permits data declarations; wiring modes would change policy. Exact raw-line or reviewed structural normalization remains open. |
| `RF-WIRE-ALIAS-ENTRY` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-ALIAS-MEMBERSHIP` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-ALIAS-MSG` | The same final assertion combines expression-shape and raw-text checks; this independent raw-text branch also needs replacement. |
| `RF-WIRE-REGISTRY-LogEntryTag` | Required source/text predicates are not translated. |
| `RF-WIRE-REGISTRY-MembershipTag` | Required source/text predicates are not translated. |
| `RF-WIRE-REGISTRY-MessageTag` | Required source/text predicates are not translated. |
| `RF-WIRE-RESERVATION` | Required documentation literal predicate is not translated. |
| `RF-WIRE-U8-SHAPES` | Call-argument shapes and exact exception parity are not implemented; ownership does not subsume this requirement. |
