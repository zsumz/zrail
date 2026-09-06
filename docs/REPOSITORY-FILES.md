# Repository-file assertions

Unreleased rc9 adds closed `[[repository.files]]` predicates. These inspect
physical paths, raw UTF-8 text, or file bytes. They do not parse Rust expressions,
interpret shell or workflow commands, or prove execution. Structured document
predicates are still a separate rc9 blocker.

```toml
[[repository.files]]
name = "root-license"
include = ["LICENSE"]
reason = "Every checkout must retain the license."
predicate = { kind = "count", minimum = 1, maximum = 1 }

[[repository.files]]
name = "retired-checker"
include = ["scripts/old-architecture-check"]
entry = "any"
reason = "The retired architecture evaluator must not return."
predicate = { kind = "exact-paths", paths = [] }

[[repository.files]]
name = "license-copies"
include = ["crates/*/LICENSE"]
reason = "Package licenses must exactly match the repository license."
predicate = { kind = "bytes-equal", other = "LICENSE" }

[[repository.files]]
name = "leading-source-contract"
include = ["crates/**/*.rs"]
exclude = ["crates/**/target/**", "crates/**/.git/**"]
reason = "The existing policy requires a raw leading documentation marker."
predicate = { kind = "literal", text = "//!", mode = "starts-with", normalization = "trim-start" }
```

These declarations belong to the contract's single `[repository]` section.
That entire section can live in one imported fragment; separate fragments do
not merge partial repository sections. The canonical identity is
`repository:file:<name>`. Names are unique and every rule requires a reason.
Existing contracts default to an empty family and retain their old behavior.
Rust API callers constructing `RepositoryContract` add `files: Vec::new()`.

## Selection and completeness

`include` is a union of canonical repository-relative literals and existing
bounded `*`, `?`, and `**` globs. Per-rule `exclude` subtracts matching paths.
Selectors cannot contain parent traversal, `.` components, duplicate separators,
or platform-dependent separators. Overlapping selectors count a path once.
Ordering has no effect. `entry` selects `file` (default), `directory`, or `any`.
The last option includes broken links and other filesystem entries. Count rules
count distinct written physical paths, not file contents or inode identities.

The shared scanner runs independently of Rust roots, source exclusions, Cargo
activation, and compilation/test reachability. Changing a source exclusion does
not remove an independently selected file from these assertions. Exact literal
paths are probed even beneath normally pruned cache directories.

Directory links are not recursively followed. A glob that could select unread
descendants of a directory link, `.git`, root `target`, or root `.zrail` fails
closed unless that subtree is explicitly excluded. A trailing `/**` exclusion
proves exclusion of descendants; excluding only the directory name does not.
Exact contained file links may be resolved and their targets are exposed and
bound. Escaping or unresolved links cannot supply file-content evidence.
The existing repository-wide symlink policy remains independently enforced.

## Closed predicates

| Kind | Semantics |
| --- | --- |
| `count` | Inclusive `minimum` and optional `maximum` over selected paths. `minimum = 0, maximum = 0` remains an active prohibition when no path exists. |
| `exact-paths` | Complete unordered `paths` set. Missing and unexpected paths fail independently of the total count. An empty set is a persistent prohibition. |
| `forbidden-names` | Literal `names`, with `part` selecting `component`, `component-stem`, `file-name`, or `file-stem`. Component stems use Rust `Path::file_stem` for directories too. |
| `literal` | Per-file raw text predicate with explicit `text`, `mode`, `normalization`, and `case`. |
| `bytes-equal` | At least one selected regular file, every file byte-for-byte equal to the required regular reference `other`. Binary data is supported. |

Name predicates default to `basis = "repository"`. The explicit `filesystem`
basis additionally inspects the canonical checkout prefix, exposes it in
coverage/explain, and binds that context into the lock. This preserves policies
that deliberately inspect absolute path components; it is not portable naming
authority inferred from a type or Cargo package.

Literal modes are `contains`, `absent`, `starts-with`, `ends-with`, `equals`, and
`exact-count`. Only exact-count accepts and requires `count`. Occurrences are
non-overlapping Rust string matches. Positive predicates require a nonempty
file selection; `absent` and exact-count zero remain valid for an empty selection.
Comments and string literals participate deliberately. These are raw predicates,
and matching command text never certifies that the command ran.

Normalization defaults to `none`. `trim-start` applies Unicode-aware Rust
`str::trim_start`; `remove-whitespace` removes `char::is_whitespace` characters.
Normalization transforms input only; whitespace-removed literals cannot contain
whitespace. Case defaults to `sensitive`; `ascii-insensitive` folds ASCII letters
only in both the literal and input. Name comparisons use the same case choices.

## Evidence, bounds, and review

File policies require a lock. Its analysis inventory binds the full observations,
selected paths and kinds, resolved file targets, inspected input hashes, and any
filesystem naming context. Even a content change that still satisfies a raw
predicate invalidates the old input binding (`LOCK-028`). No command here writes
or accepts lock authority automatically.

Coverage schema 6 adds `repository_files`, including the full policies, empty
selections, per-entry results and hashes, reference inputs, precise claim kind,
and analysis quality. Explain includes matching policies and actual observations
alongside complete scope counts; hypothetical paths do not fabricate evidence.
Literal coverage reports the full count and at most sixteen transformed-text
byte offsets, with the omitted count explicit. Samples never decide pass/fail.

Diagnostics `REP-FILE-001` through `005` identify count, exact-set, name, literal,
and byte-equality failures. `REP-FILE-006` means incomplete analysis: checks,
coverage, and lock construction fail instead of returning trusted partial data.
Reads are limited to 2 MiB per file, 64 MiB unique bytes, and 256 MiB cumulative
content work. Selection permits 8,000,000 bounded glob queries and 250,000
aggregate observations. Existing repository entry/depth and glob limits apply.
Contracts allow at most 1,024 rules, 64 selectors per rule, 20,000 exact paths,
256 forbidden names, and 16 KiB per literal. Unsupported syntax and duplicate
keys are rejected by strict contract parsing.

Protected semantic diffs classify removed guards, weakened minimum/maximum
bounds, narrower name prohibitions, and demonstrably weakened literals as grants.
Changing exact sets, exact counts, or byte references can both grant and revoke
permission; numerical decrease alone is not tightening. Selector inclusion is
interpreted according to presence versus prohibition. Unproven glob relations,
mixed quantifier changes, normalization changes, and changed justifications stay
protected as unknown. Reordering equivalent sets does not change authority.
