# Mission 10 - Trace real objects from explicit fixture roots

## Where you are

The reader can inspect supported `Node` instances during suspension. The Rust
model already handles cycles, duplicates, and deep graphs.

## The problem

The model algorithm needs to consume relationships discovered from memory and
produce the same answer as the fixture's independently declared graph.

## Observe first

Run the same two-cycle fixture with and without a connecting reference. Predict
both reachable sets before invoking native diagnostics.

## Your challenge

- [ ] Accept explicitly supplied fixture roots through the verified handoff.
- [ ] Read outgoing references from actual objects during suspension and build a
  bounded snapshot with logical IDs, roots, and discovered edges.
- [ ] Deduplicate objects before scheduling them; use an explicit work stack.
- [ ] Invoke the existing model tracer on the snapshot and compare with the
  independent fixture expectation, not with the snapshot reader itself.
- [ ] Keep native addresses distinct from model IDs; finish address-to-ID mapping
  while borrowed pointers are valid and retain only logical data after restart.
- [ ] Report roots, discovered objects, examined edges, and peak pending work.
- [ ] Fail the diagnostic pass on unsupported layouts, corruption, or exhausted
  snapshot/work budgets; report no partial success.
- [ ] Test cycles, nulls, shared children, duplicate roots, and deep chains while
  preserving the suspension/restart guarantee.

## Checkpoint

A command runs each C# fixture, reads its real links, matches the independently
expected root-reachable labels, and resumes managed execution. Changing one
managed reference changes the observed reachable set as predicted.

## Allowed shortcuts

- Copying a diagnostic snapshot into ordinary Rust collections is encouraged.
- Collection-time allocation restrictions for production marking do not apply
  to the snapshot, but its budget and failure behavior must be defined.
- Objects outside the supplied root closure need not be enumerated.

## Known debt

Reachability is relative to fixture roots. Unvisited objects must not be called
runtime garbage. Snapshot success never authorizes reclamation.

## What this unlocks

Mission 11 expands the real-object reader through additional managed fixtures.

## Hints

Introduce an adapter only where the snapshot experiment proves a useful boundary.
