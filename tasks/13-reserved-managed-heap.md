# Mission 13 - Replace scattered ZeroGC allocations

## Where you are

The managed sample runs through a bounded per-object ZeroGC. Separately, the
Rust model demonstrates checked regions, walking, tracing, sweep, and reuse.

## The problem

ZeroGC allocations occupy unrelated native ranges and cannot be walked as one
managed heap. Move to a collector-owned reservation, but keep allocation slow
and simple so the native-memory lifetime is understood before adding thread
allocation contexts.

## Observe first

Record ZeroGC's allocation ranges and current managed-address/write-barrier
configuration. Write a small platform experiment that reserves more address
space than it commits, touches only committed pages, decommits them, and commits
them again.

## Your challenge

- [ ] Add an owning Windows reservation that distinguishes reserve, commit,
  decommit, and release and preserves native error codes.
- [ ] Validate page-aligned subranges with checked arithmetic and release exactly
  the owned reservation on drop without panicking.
- [ ] Keep the platform API injectable so range and allocation policy tests do
  not require Windows calls.
- [ ] Replace per-object native allocation with a simple locked bump frontier in
  committed portions of the reservation. Commit more pages on demand.
- [ ] Publish correct managed bounds and write-barrier/card state for the pinned
  runtime; reject uncertainty rather than neutralizing the barrier silently.
- [ ] Keep the mission 03 smoke and mission 04 bounded exhaustion scenario
  deterministic, with reserved and committed byte counters.
- [ ] Test invalid ranges, reserve/commit failure injection, guard pages, and
  release after partial initialization.

## Checkpoint

`LoaderSmoke` still exits normally under the custom GC, every returned object is
aligned inside an owned committed range, committed-byte accounting reconciles,
and the configured limit fails without memory corruption.

## Allowed shortcuts

- All allocation may use one global lock and one active range.
- Collection and real heap walking are not required.
- The native layout does not need to reuse the model's teaching headers.

## Known debt

Contention is expected, abandoned allocation tails are not yet walkable managed
free objects, and object boundaries still cannot be reconstructed reliably.

## What this unlocks

Mission 14 can measure global-allocation contention and introduce allocation
contexts as a focused allocator refactor.

## Hints

Avoid creating long-lived Rust slices across partially committed memory. State
explicitly whether the reservation owner is `Send` or `Sync` and why.
