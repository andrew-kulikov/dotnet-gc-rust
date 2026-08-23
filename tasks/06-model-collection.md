# Mission 06 - Remove dead graph objects

## Where you are

The model computes reachability in a plain Rust graph but leaves the graph
unchanged.

## The problem

A collector must distinguish tracing from reclamation and define what happens
when either phase encounters corruption. Mutating the graph while discovering
reachability can accidentally delete data still needed by the traversal.

## Observe first

Create a graph containing a live cycle, a dead cycle, and an edge from the live
component to the dead component. Predict the survivors, run marking, and inspect
which mutation order would make that prediction unreliable.

## Your challenge

- [ ] Introduce explicit model phases sufficient to prevent allocation or graph
  mutation through safe APIs while collection is in progress.
- [ ] Run reachability without deleting objects, then reclaim every object not
  in the completed live set.
- [ ] Return allocated-before, live, reclaimed, and allocated-after counts and
  make them reconcile.
- [ ] Leave the model in an allocatable state after success and in a defined,
  inspectable state after a detected corruption.
- [ ] Test repeated allocate/collect sequences, all-live, all-dead, dead cycles,
  and collection of an already empty graph.

## Checkpoint

Generated sequences match a simple reference model after every complete
collection, and invalid phase transitions are rejected by the public API.

## Allowed shortcuts

- Reclamation may be `HashMap::retain`.
- No byte storage, free records, fragmentation, or object reuse is required.

## Known debt

Deleting a map entry does not teach the collector how dead bytes remain
walkable or become reusable.

## What this unlocks

Mission 07 can insert handle semantics between marking and reclamation while the
phase boundary is still easy to inspect.

## Hints

Keep “compute live set” and “mutate storage” as separate operations even if the
model could combine them in fewer lines.
