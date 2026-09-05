# Approved policy decisions

## RC9-DECISION-RF-PRIVATE-NAMES

On 2026-09-05 the user instructed: "Just get everything needed in zrail done.
Privat ename stuff will be removed from rafter."

This approves retiring Rafter's private-name policy and its scan-specific
machinery. It closes `RF-PRIVATE-NAME-PATTERNS`: the externally supplied pattern
set is no longer an input needed for zrail replacement qualification. The frozen
source remains `e99b43c1b3fb3c94dd626effccee8420016594f2`, with
`scripts/private-name-scan` SHA-256
`3d648d3358560b6943b4cb0d48eda6f926bab84cd53bc6b88524803e05177a0b`.

This decision does not retire general naming, vocabulary, license, publication,
package inventory, or provenance requirements. Independent build, package, and
artifact verification programs remain execution evidence. The scan's proof that
its own globs cover shipped files belongs to the retired scan, not to a new
claim that stock zrail has executed Cargo.

No downstream deletion is performed here. The assertion inventory must still
identify scan-specific predicates, detector tests, and invocations and link their
retirement to this decision. Existing verified-predicate counts are unchanged;
recording a decision is not detector parity evidence.
