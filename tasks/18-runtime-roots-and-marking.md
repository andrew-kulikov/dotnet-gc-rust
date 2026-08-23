# Mission 18 - Mark from real runtime roots

## Where you are

The execution engine can be stopped and reliably restarted around a verified
heap walk. Real outgoing object references can be enumerated.

## The problem

The model accepts logical roots, while CoreCLR reports stack and runtime roots
through callbacks valid only during suspension. Adapt those callbacks without
letting transient pointers escape their phase, then determine live and dead real
objects without reclaiming anything yet.

## Observe first

Create a managed fixture that retains known objects independently through a
local stack variable, a static, a thread-static, and an exception path. During a
read-only suspension, log root kind/flags and verify those known roots appear.

## Your challenge

- [ ] Adapt `GcScanRoots` callbacks into a non-owning visit operation whose
  pointers cannot outlive suspension.
- [ ] Validate each root against managed/frozen address rules before using it.
- [ ] Record root flags needed by later pinning and interior-pointer work without
  pretending to support their full semantics.
- [ ] Seed side mark metadata from validated roots and trace outgoing slots with
  an explicit work stack.
- [ ] Diagnose known-dead objects after marking but do not alter or reuse their
  memory.
- [ ] Clear or version mark state for the next cycle and report root, object,
  edge, work-stack, and phase-timing counters.
- [ ] Restart the execution engine on corruption, work-stack failure, or any
  injected mark error.

## Checkpoint

Known-live objects from every claimed root source remain marked across repeated
cycles, known-dead objects are reported dead, deep graphs do not recurse, heap
walking still succeeds, and every run resumes managed execution.

## Allowed shortcuts

- Reclamation and reuse are forbidden in this mission.
- Unsupported root flags may abort the collection safely.
- Expensive address validation is encouraged.

## Known debt

Dead managed objects still contain their original method-table pointers and
remain indistinguishable from allocated objects to the next heap walk.

## What this unlocks

Mission 19 can mutate only the already diagnosed dead ranges into valid free
objects while reusing suspension and marking as trusted phases.

## Hints

Keep runtime addresses distinct from model object IDs. A root callback lends a
slot for the suspended phase; it does not transfer Rust ownership.
