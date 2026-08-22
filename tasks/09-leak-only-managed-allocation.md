# Iteration 09 — Leak-only managed allocation

## Goal

Run a managed program using Rust-owned allocation segments without reclaiming memory.

## Assignment

- [ ] Reserve the collector heap and commit segments on demand.
- [ ] Implement thread allocation contexts and aligned bump allocation.
- [ ] Provide stable-address handle storage sufficient for startup and the smoke workload.
- [ ] Initialize valid heap bounds, card-table state, and required write-barrier data.
- [ ] Extend `samples/LoaderSmoke` with bounded small, large, pinned, and finalizable allocations.
- [ ] Report allocation and commitment counters on shutdown.

## Constraints

- This iteration intentionally leaks managed objects; do not start collection.
- The sample must use a bounded allocation volume below the reserved limit.
- Do not neutralize write-barrier state in a way that prevents the later regional collector.

## Acceptance criteria

- The sample exits normally under the custom GC on the pinned runtime.
- Every allocation is aligned, inside an owned committed range, and reflected in counters.
- Exhaustion follows a deterministic, documented failure path rather than corrupting memory.
- The same sample still runs under the stock GC.

## Hints

Prefer a deliberately slow but obviously correct allocation path before adding allocation contexts. Log ranges and sizes, never raw object contents.

## Agent review focus

Ask the agent to audit allocation-context tail handling, segment lifetime, heap bounds, commitment accounting, and all out-of-memory paths.
