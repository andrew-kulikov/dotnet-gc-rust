# Mission 14 - Refactor the allocator around thread contexts

## Where you are

Managed objects come from one reserved, lazily committed range through a slow
locked allocator.

## The problem

Every allocation currently contends on shared state. CoreCLR provides thread
allocation contexts so the common path can bump `alloc_ptr` without entering the
collector. Using them changes how unused tails and heap frontiers must be owned.

## Observe first

Run a deterministic two-thread allocation workload and record lock acquisitions,
allocation requests, and elapsed time. Trace the pinned runtime's allocation
context calls and fields before changing the allocator.

## Your challenge

- [ ] Give a thread a bounded, zeroed allocation context from the reserved heap
  and let valid fast-path allocations advance only that context.
- [ ] Refill or replace an exhausted context through a synchronized slow path.
- [ ] Define who owns an active context's unused tail and how it is retired on
  thread/context transition without overlapping another context.
- [ ] Route specially flagged or oversized requests through an explicit slow
  path and fail unsupported flags by name.
- [ ] Preserve alignment, committed-range, bounds, and counter invariants from
  mission 13 under concurrent allocation.
- [ ] Compare the two-thread workload with the previous global-lock baseline.
  Correctness is required even if performance does not improve yet.

## Checkpoint

The multithreaded bounded workload exits normally, object ranges never overlap,
fast-path allocations do not acquire the global allocator lock, and every byte
handed to or left in a context is accounted for.

## Allowed shortcuts

- Context refill may remain coarse-grained and locked.
- Retired tails may be recorded diagnostically until mission 15 defines a real
  managed free-object representation.
- No collection is required.

## Known debt

Without managed object-size decoding and valid tail records, the collector still
cannot walk the stopped heap from bytes alone.

## What this unlocks

Mission 15 can make actual managed allocations self-walkable by deriving their
sizes and representing retired tails correctly.

## Hints

Treat publication and retirement of a context as state transitions, not merely
pointer assignments. Keep the slow path easy to verify.
