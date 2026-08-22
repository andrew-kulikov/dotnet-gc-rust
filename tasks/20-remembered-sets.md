# Iteration 20 — Write barrier and remembered sets

## Goal

Discover references from non-victim regions into victim regions without scanning the full non-victim heap.

## Assignment

- [ ] Turn write-barrier card updates into region-local dirty-card information.
- [ ] Scan dirty cards conservatively for possible references into selected victims.
- [ ] Define when cards are cleared, retained, or rescanned.
- [ ] Add cross-region mutation workloads and a slow full-edge oracle.
- [ ] Measure dirty-card density and abandon partial collection when scanning it is more expensive than a full collection.

## Constraints

- Missing one incoming edge is a correctness failure; false positives are acceptable and measurable.
- Card state updates must respect the runtime's concurrency and memory-ordering contract.
- Do not add a custom managed write barrier outside supported CoreCLR integration.

## Acceptance criteria

- Every oracle edge into a victim is found after arbitrary supported mutations.
- Concurrent mutation stress does not lose dirty state under the documented suspension model.
- Card clearing cannot erase a mutation that must be observed by the next partial collection.
- Dense-card workloads choose explicit fallback.

## Hints

Write down the exact mutator/collector happens-before relationship. Sticky cards are often a reasonable first implementation.

## Agent review focus

Ask the agent to inspect races around card clearing, boundary-spanning objects, false-negative tests, and the full-collection fallback threshold.
