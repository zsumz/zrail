# Signed rc9 history

On 2026-09-07, all 118 commits after the published rc8 base were rewritten as
PGP-signed `zsumz <shawn@zsumz.com>` commits. Messages use `type: subject`, with
no body or coauthor trailers. Every original tree, parent order, author date,
and committer date was preserved and verified. Signatures use fingerprint
`B58439871CD2A7275B20CC19EC8E4D26598A0373`.

The original tip `66b5b57415a967de5703a155f35cad8f74159b01` corresponds to signed
tip `545b671de3f6288af95d4ae7fb7ffd12267dd8db`. Both have tree
`7276b98d1a1f497a2bce2269a19f6ad86d3b2dcf` and descend from the unchanged rc8
base `5a368379360104ca19745326cfcef48d22a6452b`.

[The complete correspondence map](HISTORY-REWRITE.json) records each old and
new commit, its common tree, and both subjects. To recover the source used by
a historical evidence record, find its `old_commit` in that map and inspect
the associated `new_commit`. Verify its tree against the recorded `tree`.

Existing evidence retains its original commit identifiers, compiler and binary
identities, input hashes, and results. The mapping proves source equivalence;
it does not claim that old executions ran under a new commit identity or that
later code changes inherit their qualification.

The original history is also preserved locally by the branch
`backup/rc9-guardrail-coverage-unsigned-20260907` and a verified Git bundle on
the zdev volume. The feature branch was replaced using an exact remote lease.
The published rc8 tag and protected main were not rewritten.
