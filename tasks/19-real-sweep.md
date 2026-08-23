# Mission 19 - Sweep dead managed objects without reuse

## Where you are

A stop-the-world mark phase identifies live and dead objects in the real managed
heap and always restarts execution. Dead bytes are not yet changed.

## The problem

After a collection, the heap must remain walkable. Simply clearing a dead object
destroys the metadata used to find the next object. Convert dead ranges into the
pinned runtime's valid free-object representation before attempting any native
reuse policy.

## Observe first

Choose one fixture with a known dead object between two known-live objects.
Record all three ranges before collection and run the current marker without
mutation. Derive the minimum valid free-object size and layout from pinned
runtime source.

## Your challenge

- [ ] Sweep every walked record while suspended, preserving live objects and
  replacing dead objects with valid managed free objects covering exactly their
  original ranges.
- [ ] Define behavior for a dead range too small to encode independently; do not
  invent an unwalkable fragment.
- [ ] In debug builds, poison only bytes permitted by the free-object layout and
  immediately re-walk the entire affected heap.
- [ ] Reconcile allocated-before, live, dead, free-after, padding/tails, and total
  committed walkable bytes.
- [ ] Inject corruption and verification failures during sweep and preserve the
  mission 17 restart guarantee.
- [ ] Add repeated forced collections for cycles, deep graphs, multiple threads,
  exceptions, and rapidly dying objects without returning free objects to the
  allocator.

## Checkpoint

Known-live objects survive, known-dead ranges become valid free objects, the
entire heap walks after every collection, byte accounting reconciles, and managed
execution resumes on all tested success and failure paths.

## Allowed shortcuts

- The allocator must not reuse real free objects yet.
- Adjacent real free objects need not be coalesced.
- Runtime features not demonstrated by the current fixtures may remain explicit
  unsupported cases.

## Known debt

This is the first real collector checkpoint, not a usable baseline release.
Runtime handle completeness, interior pointers, frozen memory, finalization,
real-heap reuse, fragmentation policy, and long soak stability are intentionally
unscheduled until this mission passes.

## What this unlocks

After review, use observed failures and heap invariants to write the next batch
of missions. Do not copy the old phase-5 list unchanged: decide ordering from
the runtime features and reuse problems actually encountered here.

## Hints

Keep mutation after a complete successful mark. Verify before restart even when
the verification is expensive. Never let a plausible diagnostic substitute for
a complete byte-accounting equation.
