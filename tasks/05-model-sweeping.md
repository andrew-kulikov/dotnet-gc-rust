# Iteration 05 — Model sweeping and reuse

## Goal

Reclaim unmarked model objects and reuse their storage while keeping every region walkable.

## Assignment

- [ ] **Sweep the walkable representation from left to right.** Decode every allocated or
  free record up to the frontier. Preserve marked objects, clear their mark for the next
  cycle, and replace each unmarked object with a valid free-object record covering exactly
  the same bytes. Do not rely on a separate list of allocations to find dead objects.
- [ ] **Merge physically adjacent free records.** When two neighboring records are free,
  replace them with one record whose checked size covers both. Test a free object with a
  free neighbor on the left, right, both sides, and across repeated collections; never
  merge across a region boundary.
- [ ] **Index reusable blocks by size class.** Maintain segregated free lists whose buckets
  represent documented size ranges. Each reusable free record must appear in exactly one
  bucket, and allocation should search suitable buckets before extending the bump
  frontier. The heap bytes remain authoritative; lists are an acceleration structure that
  the verifier can rebuild and compare.
- [ ] **Split only when both resulting records are valid.** An allocation may consume all
  of a selected free block or split off a remainder large enough to encode a free header
  and satisfy the model's alignment. If the remainder is too small, consume the entire
  block rather than creating an unwalkable fragment. Any failure must restore the list
  entry and original free record.
- [ ] **Define and reconcile space metrics.** Report bytes occupied by surviving objects,
  bytes made dead in this cycle, bytes currently available for reuse, and unusable
  fragments/internal waste. Add a verifier equation so overlapping or missing bytes make
  the test fail instead of producing plausible-looking counters.

## Constraints

- Free-list metadata and the walkable heap representation must agree after every operation.
- Survivors do not move.
- Allocation failure must not lose a free block or corrupt a list.

## Acceptance criteria

- Reachable objects survive; unreachable objects become reusable.
- Walking after arbitrary allocate/collect cycles covers each byte exactly once up to the frontier.
- Free lists contain every reusable free object exactly once and contain no allocated object.
- Tests cover exact fit, split, too-small remainder, coalescing on both sides, and repeated reuse.

## Hints

Write a slow heap verifier before optimizing lists. Run it after every operation in property tests and debug builds.

## Agent review focus

Ask the agent to reconcile the walker, free lists, byte counters, and allocation frontier from first principles on randomized sequences.
