# Iteration 13 — First real mark and sweep

## Goal

Perform a complete stop-the-world collection of real managed objects without reusing reclaimed space yet.

## Assignment

- [ ] Feed runtime roots and `GCDesc` reference slots into the iterative marker.
- [ ] Store real-heap mark state in side metadata.
- [ ] Sweep dead objects into valid walkable free objects.
- [ ] Clear reclaimed memory in debug builds and verify the entire heap after collection.
- [ ] Add forced-collection workloads for cycles, deep graphs, multiple threads, exceptions, and rapidly dying objects.
- [ ] Report phase timings and byte counters.

## Constraints

- Survivors remain at their original addresses.
- Reclaimed objects are not returned to allocation in this iteration.
- The execution engine must restart even when verification detects corruption.

## Acceptance criteria

- Known-live graphs survive repeated forced collections and known-dead objects are detected as dead.
- Heap walking succeeds after every tested collection.
- Byte accounting reconciles allocated, live, and dead space.
- Stress runs do not depend on debug logging for correctness.

## Hints

Reuse algorithms from `gc-core`, but keep runtime address validation separate from model object IDs. Add expensive verification before optimizing pause time.

## Agent review focus

Ask the agent to look for missing root categories, stale mark bits, work-stack pointer invalidation, and error paths that resume with a partially mutated heap.
