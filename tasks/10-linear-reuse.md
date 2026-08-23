# Mission 10 - Reuse one dead byte range

## Where you are

The model heap has typed region ranges, self-describing allocated records, a
walker, and marking. Collection still cannot reuse bytes.

## The problem

Removing an object from a logical graph is not enough in a byte heap. The walker
must still cross the dead object's bytes, and a later allocation needs a way to
recognize that range as available.

## Observe first

Allocate three records, keep the first and third live, and collect the middle
one. Overwrite the middle bytes with zeros and run the walker to demonstrate why
dead space needs its own valid record format.

## Your challenge

- [ ] During a forward sweep, preserve marked records, clear their marks for the
  next cycle, and replace each unmarked record with a valid free record covering
  exactly the same byte range.
- [ ] Extend the walker so allocated, padding, and free records all make checked
  forward progress.
- [ ] Allocate from the first sufficiently large free record found by a linear
  heap walk before extending the bump frontier.
- [ ] Support exact-fit reuse and either consume or safely encode any remainder.
  A failed reuse attempt must leave the original free record intact.
- [ ] Reconcile live, newly reclaimed, currently reusable, frontier, and padding
  bytes after every collection.

## Checkpoint

An allocate/collect/allocate scenario reuses the dead middle range, preserves
both live neighbors, and remains fully walkable. Random sequences match a slow
reference model and byte-accounting equation.

## Allowed shortcuts

- Reuse may linearly scan the entire heap.
- Adjacent free records need not be merged yet.
- There is no free-list index.

## Known debt

Linear search becomes increasingly expensive, and separate adjacent holes may
fail a request that their combined space could satisfy.

## What this unlocks

Mission 11 can measure those two limitations and introduce coalescing or size
buckets only where the workload demonstrates value.

## Hints

Treat heap bytes as authoritative. Any later index should be disposable and
rebuildable from a verified walk.
