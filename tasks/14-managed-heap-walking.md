# Mission 14 - Walk the stopped managed heap from bytes

## Where you are

Supported sizes and reference slots are known, and managed objects occupy a
collector-owned range with a simple allocation frontier.

## The problem

Root tracing only visits reachable objects. Sweep also needs a complete,
authoritative inventory of allocated and free records.

## Observe first

Allocate mixed supported objects and compare a byte walk with a diagnostic
allocation inventory. Include alignment gaps and any unused context tails.

## Your challenge

- [ ] Decode records from the range start to its exact stopped frontier using
  the pinned runtime's size, alignment, and free-object rules.
- [ ] Extend shape support for every record encountered in the declared workload;
  unsupported records stop verification before any collection mutation.
- [ ] Represent padding and unused tails unambiguously. Close any active contexts
  using their actual pinned-runtime contracts before walking.
- [ ] Check metadata provenance, lengths, arithmetic, containment, and progress.
- [ ] Treat the byte walk as authoritative; use the allocation registry only as
  an independent comparison and remove it as a correctness dependency.
- [ ] Feed walked object starts and boundaries into address validation and the
  existing tracing diagnostics.
- [ ] Verify the entire stopped heap and resume on success or bounded failure.

## Checkpoint

Mixed managed allocations walk exactly to the known frontier and match the
independent inventory. Corrupt synthetic inputs fail deterministically; injected
verification failures still attempt restart exactly once.

## Allowed shortcuts

- One range and slow allocation remain sufficient.
- Free-object encoding may be needed only for padding/tails at this point.
- A side object-start index may be rebuilt from each verified walk.

## Known debt

A walk inventories objects but does not establish that every liveness source
was processed. Runtime handle semantics still constrain reclamation.

## What this unlocks

Mission 15 establishes handle ordering in the model before runtime integration.

## Hints

An object address must resolve to a validated object start or a separately
supported interior-reference rule; containment alone is insufficient.
