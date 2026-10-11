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

- [x] Introduce explicit model phases sufficient to prevent allocation or graph
  mutation through safe APIs while collection is in progress.
- [x] Run reachability without deleting objects, then reclaim every object not
  in the completed live set.
- [x] Return allocated-before, live, reclaimed, and allocated-after counts and
  make them reconcile.
- [x] Leave the model in an allocatable state after success and in a defined,
  inspectable state after a detected corruption.
- [x] Test repeated allocate/collect sequences, all-live, all-dead, dead cycles,
  and collection of an already empty graph.

## Checkpoint

Generated sequences match a simple reference model after every complete
collection, and invalid phase transitions are rejected by the public API.

Run `cargo test -p gc-rust` for the runtime and compile-fail checks. Run
`cargo run -p gc-rust --example graph_collection` for an observable collection:
the first sweep reports `before=4, live=2, reclaimed=2, after=2`; allocation
resumes, then collecting with no roots reports `before=3, live=0, reclaimed=3,
after=0`. See [the example notes](../crates/gc-rust/examples/README.md).

The implementation uses borrowed `Marking` and `Marked` phases. The compiler
rejects allocation, edge mutation, or a second collection overlapping the
exclusive borrow, sweep before successful marking, and reuse of a swept result.
Dropping either phase cancels without mutation. Marking errors release an
unchanged, inspectable graph; corruption is preserved and still rejected on
the next attempt. Thirty-two generated sequences each check 100 collections
against a separately stored, fixed-point reference model.

## Allowed shortcuts

- Reclamation may be `HashMap::retain`.
- No byte storage, free records, fragmentation, or object reuse is required.

## Known debt

Deleting a map entry does not teach the collector how dead bytes remain
walkable or become reusable.

## What this unlocks

Mission 07 expresses the same small graph scenarios in managed code and checks
them against the model before reading actual runtime objects.

## Hints

Keep “compute live set” and “mutate storage” as separate operations even if the
model could combine them in fewer lines.
