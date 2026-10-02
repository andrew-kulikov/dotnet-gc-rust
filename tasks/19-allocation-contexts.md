# Mission 19 - Add thread allocation contexts to the collector

## Where you are

A slow synchronized allocator supports verified walk, mark, and sweep cycles
for the declared workload. Freed ranges remain unused.

## The problem

Thread contexts reduce allocation contention but introduce independently owned
frontiers and unused tails that collection must account for.

## Observe first

Measure lock acquisitions and allocation work in a supported two-thread workload.
Trace the pinned runtime's allocation-context publication and retirement calls.

## Your challenge

- [ ] Hand threads bounded zeroed contexts from owned committed storage and
  support valid fast-path allocation without the global allocator lock.
- [ ] Refill exhausted contexts through a synchronized slow path.
- [ ] Define active-tail ownership, publication, and retirement without overlap.
- [ ] At suspension, close contexts and encode tails as valid walkable records
  before invoking the established heap verifier and marking pipeline.
- [ ] Keep oversized or specially flagged allocations on an explicit supported
  slow path; unsupported flags fail by name.
- [ ] Preserve checked bounds, accounting, roots, handles, and restart invariants
  under concurrent allocation and repeated supported collections.
- [ ] Compare measured lock work and validate failed refill/retirement behavior.

## Checkpoint

The bounded multithreaded workload allocates without overlap and survives
repeated verified collections. Fast-path allocation avoids the global allocator
lock, and every context byte is accounted for after retirement.

## Allowed shortcuts

- Context refill may remain coarse-grained and locked.
- Reuse of swept free records is not required.

## Known debt

Memory still grows to the configured frontier limit. Reuse policy and broader
runtime support require further measured experiments.

## What this unlocks

Measured allocation and fragmentation behavior can motivate reuse work, with
the storage laboratories supplying isolated correctness experiments.

## Hints

Optimize the common path only after publication and stopped-heap invariants
are observable in tests.
