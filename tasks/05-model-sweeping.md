# Iteration 05 — Model sweeping and reuse

## Goal

Reclaim unmarked model objects and reuse their storage while keeping every region walkable.

## Assignment

- [ ] Convert dead objects into valid free objects during a forward sweep.
- [ ] Coalesce adjacent free objects.
- [ ] Maintain segregated free lists and allocate from them before the bump frontier.
- [ ] Split a free object only when the remainder can be represented safely.
- [ ] Track live, reclaimed, reusable, and fragmented bytes.

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
