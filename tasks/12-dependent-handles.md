# Mission 12 - Let a handle chain require a fixed point

## Where you are

The model has tracing, reclamation, strong/weak handles, and reusable byte
storage. It has no dependent-handle semantics.

## The problem

A dependent handle makes its secondary live only when its primary is already
live. One pass over handles is insufficient when an earlier secondary becomes a
later primary.

## Observe first

Add handles `A -> B` and `B -> C`, root only `A`, and process the handles once in
the order that sees `B -> C` first. Record why `C` remains incorrectly dead.

## Your challenge

- [ ] Add dependent handles with distinct primary and secondary targets. The
  secondary must never keep its own primary alive.
- [ ] After ordinary tracing, process live primaries and trace newly discovered
  secondaries; repeat complete passes until a pass discovers nothing new.
- [ ] Place weak clearing and reclamation after the fixed point.
- [ ] Report fixed-point passes, handles examined, and new objects discovered.
- [ ] Build an independent simple oracle and test chains, cycles, null targets,
  unreachable pairs, mixed strong/weak handles, stale IDs, and order changes.

## Checkpoint

All handle orders produce the oracle's live set, the adversarial chain keeps
`C` alive, an unreachable dependent pair dies, and no weak/dependent lookup
exposes reclaimed storage.

## Allowed shortcuts

- Repeated full scans of the dependent-handle collection are expected.
- No attempt to optimize passes or mirror CoreCLR storage layout is needed.

## Known debt

The model captures liveness semantics only. Runtime handle kinds, stable native
addresses, finalization ordering, and concurrency remain future discoveries.

## What this unlocks

The runtime-independent experiments now cover the core semantic and storage
mechanisms. Mission 13 can replace ZeroGC's scattered allocations with a real
reserved address range while keeping the model as a differential oracle.

## Hints

Make each pass return whether it discovered anything. Do not assume iteration
order will remain stable as the handle container grows.
