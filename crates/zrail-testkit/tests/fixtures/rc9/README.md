# Frozen rc9 audit data

This is a **partial assertion review**, not certified guardrail replacement.
No downstream checker is removable on the strength of this directory.

- `snapshots.json` pins the implementation base and every consumer Git tree.
- `census.json.gz` is the deterministic syntax census of **all tracked files**,
  including separate workspaces and invalid detector fixtures. The decompressed
  JSON bytes are canonical; `census-summary.json` gives counts and their SHA-256.
- `assertions.json` records 672 reviewed assertion instances, including separately
  instantiated capability tokens and lock provenance assertions. It binds each
  assertion to a census identity, source digest, selection, matching semantics,
  cardinality, exceptions, disposition, and independent implementation and
  verification states. Its 113 open assertion IDs are release blockers, alongside
  the unfinished discovery work; 559 instances have verified replacement evidence.
- `facade-origins.json` binds two byte-exact extracted predicates and the imported
  `kafkars-facade-invalid.rs.txt` fixture to the frozen sources.
- The budget, size-selection, file, and metadata origin manifests bind further
  byte-exact predicates and input copies. Metadata includes the complete original
  assertion body, read/parse helpers, and all 11 physical fixture inputs. `key-sets-origins.json` binds five unchanged
  dependency assertion bodies, their manifest readers, and six immutable inputs;
  130 differential fixtures qualify the exact/subset and type-projection semantics.
  `dependency-fields-origins.json` binds six further unchanged assertion bodies,
  sharing those six inputs; 151 fixtures qualify 35 authored-field assertions. `provenance-origins.json`
  binds the original no-override body, helper conjunctions, and four inputs;
  161 fixtures qualify 36 authored provenance assertions and parser preconditions.

Candidate IDs have the form `repository:path:line:column:kind`, with one-based
physical coordinates and a normalized syntax digest. They are stable within the
frozen input. An assertion ID can instantiate one candidate for a specific
registry value. A candidate linked by one assertion can still contain additional
unreviewed helper branches or policy instances. Unlinked candidates and non-Rust
files are pending review, never automatically classified as behavioral tests.

Five detector-negative facade assertions have predicate and diagnostic evidence.
The two live facade assertions have implemented predicates but still lack
complete consumer-selection qualification. No full repository is qualified.

The one Rust parse boundary is Kafkars' deliberately malformed
`tests/fixtures/invariant_registry/src/invalid_test.rs`; it remains in the census.
Its enclosing detector and expected failure still need assertion-level review.

Integrity and reproducibility checks:

```sh
python3 scripts/rc9-inventory-check
python3 scripts/rc9-inventory-check --census /path/to/repeated-census.json
python3 scripts/rc9-inventory-check --require-complete
```

The last command **must fail** while inventory is incomplete. Integrity success
does not establish release readiness. Reproduction commands and limitations are
in `docs/rc9/QUALIFICATION.md`. These scripts are trusted qualification tools;
stock zrail does not execute them or any downstream evaluator during analysis.

The descriptions below retain each family's initial evidence scope; consult
`assertions.json` and `docs/rc9/CHECKPOINT.md` for current verification status.
The new registry structures/loader and 249 source/27 native cases are focused
fixtures only. They expose `RC9-NATIVE-TRAVERSAL-CONTRACT`; neither registry
assertion has repeated, bound replacement evidence yet.

- `lock-packages-origins.json` binds the unchanged complete protocol lock test
  and helpers. Twelve frozen inputs and 50 fixtures qualify 20 exact whole-lock
  assertions; these graph-only inputs do not establish source completeness.

- `raw-dependency-origins.json` binds the exact raw lock-name extractor, CI and
  attributes assertions, read helpers, and four inputs. 157 fixtures qualify
  16 assertions without interpreting Cargo, YAML, shell, or workflow execution.

- `qualification-origins.json` binds ten complete frozen qualification test
  bodies, their image constants/read helper, and 24 inputs. 923 fixtures qualify
  125 raw-text/read assertions, including first-marker order and intervals and
  complete image-line suffix allowlists. The unenforced-zrail prohibition remains
  a separate cutover blocker; actual qualification and behavioral runners remain.

- `route-sources-origins.json` binds the mixed route test, eleven exact included
  source inputs and its unchanged raw-text assertions. All 28 census candidates
  in this file are accounted for by 31 assertion instances: sixteen source/input
  contracts and fifteen retained runtime assertions. The translated six stock
  file rules have focused fixtures; committed repeated evidence and the runtime
  execution receipt remain unverified.

- `audited-text-origins.json` binds the complete frozen Rafter source-boundary,
  private-name script and release-document inputs. The ledger records 21 manual
  text spans with UTF-8 byte offsets and digests separately from parsed Rust
  candidates. Source/supporting-helper links cover all 14 candidates in the
  mixed source-boundary guard. Its 41 retained and 25 retirement instances are
  inventoried but not counted as verified replacement evidence.
