# Laboratory 05 - Let a handle chain require a fixed point

## Where you are

The graph model from missions 06 and 15 supports tracing, reclamation, and
strong/weak handles. This laboratory requires no byte heap or native runtime.

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

The fixed-point oracle can guide a runtime dependent-handle mission when a
managed fixture demonstrates the need. Native slot lifetimes and callback
contracts must be validated separately.

## Hints

Make each pass return whether it discovered anything. Do not assume iteration
order will remain stable as the handle container grows.
