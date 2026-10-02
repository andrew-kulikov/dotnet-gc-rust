# Mission 18 - Reclaim real objects after complete marking

## Where you are

A completed mark identifies live and dead ranges for the explicitly supported
runtime workload and prepares weak clearing without changing managed memory.

## The problem

Dead objects must become valid free records so the heap remains walkable.
Mutation must follow complete liveness analysis in the same suspended window.

## Observe first

Place a known-dead object between two retained objects. Derive the exact valid
free-object representation and minimum size from pinned runtime source.

## Your challenge

- [ ] Require a fresh completed mark from the current uninterrupted suspension.
  On incomplete marking, leave weak slots and object bytes unchanged.
- [ ] Apply supported weak clearing before reclaiming targets.
- [ ] Preserve live records and encode dead ranges as valid managed free records
  covering exactly their original bytes; reject unencodable ranges before writes.
- [ ] Preflight sweep inputs and define failures after mutation explicitly; do
  not claim rollback or resumability when heap invariants cannot be restored.
- [ ] Re-walk and verify affected ranges before managed execution resumes.
- [ ] Reconcile live, dead, free, padding/tail, frontier, and committed bytes.
- [ ] Inject failures before and during sweep. Recoverable exits attempt restart
  exactly once after restoring heap invariants; unrecoverable corruption follows
  an explicit terminal policy instead of resuming a known-invalid heap.
- [ ] Run repeated bounded cycles with live/dead cycles, deep graphs, required
  handles, exceptions, and each supported mutator configuration.

## Checkpoint

Known-live objects survive, supported weak targets clear at the right phase,
and known-dead ranges become valid free objects. Successful collections verify
and reconcile bytes before continuing managed execution. Incomplete marking
never mutates the heap; failure outcomes are explicit and tested.

## Allowed shortcuts

- The allocator must not reuse reclaimed ranges yet.
- Coalescing and free-space indexes are unnecessary.
- The supported runtime feature set remains explicit and bounded.

## Known debt

The bump frontier still grows. Finalization, dependent handles, interior/frozen
cases, and additional runtime features need their own verified capabilities.

## What this unlocks

Mission 19 measures allocation contention and integrates thread contexts with
an already working walk, mark, and sweep cycle.

## Hints

Reserve any sweep bookkeeping before mutation. Mark completion is a phase-local
capability, not a reusable boolean.
