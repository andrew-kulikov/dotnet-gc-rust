# Mission 11 - Extend object and reference decoding by fixture

## Where you are

Tracing real `Node` graphs matches the model through a diagnostic snapshot.

## The problem

Runtime root traversal will encounter more than the fixture class. Each claimed
shape needs independently checked boundaries and reference enumeration.

## Observe first

Add one fixture at a time: inherited class, boxed value, string, reference array,
and value-type array containing nested references.

## Your challenge

- [ ] Derive base size, component size, lengths, alignment, and necessary
  descriptor forms from the pinned runtime source.
- [ ] Check signed conversions, products, sums, bounds, and forward progress
  before reading an object or descriptor slot.
- [ ] Extend positive and required negative `GCDesc` forms only with fixtures
  that demonstrate their semantics.
- [ ] Compare object sizes and exact reference slots with independent fixture
  expectations or a diagnostic tool.
- [ ] Preserve null handling, cycle detection, snapshot budgets, and restart on
  failure for mixed graphs.
- [ ] Generate malformed decoder inputs and require bounded failure.
- [ ] Maintain an explicit supported-shape table; unsupported layouts make a
  traversal incomplete rather than silently hiding outgoing references.

## Checkpoint

Every claimed shape has passing size and slot-enumeration fixtures. Mixed
supported graphs match their expected reachable sets. Unknown layouts and
corrupt input produce explicit incomplete/error outcomes and resume execution.

## Allowed shortcuts

- Support one pinned Windows x64 runtime.
- Full heap walking and free-object encoding are not required.
- DAC may aid verification but must not be required for correctness.

## Known debt

Shape coverage is explicit and may be insufficient for a complete runtime-root
walk. No object may be reclaimed on the strength of these diagnostics.

## What this unlocks

Mission 12 uses runtime root callbacks to discover which additional cases the
actual managed workload requires.

## Hints

Keep shape decoding outside the generic reachability algorithm.
