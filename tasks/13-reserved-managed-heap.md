# Mission 13 - Allocate managed objects in owned storage

## Where you are

Suspended diagnostics already trace supported real objects and report root
coverage. ZeroGC still owns a separate native block for each allocation.

## The problem

A collector-owned range makes full byte walking and eventual reclamation
possible. Allocation policy should stay simple while storage ownership changes.

## Observe first

Record the current fixture trace results, native ranges, and write-barrier
configuration. Exercise Windows reserve, commit, decommit, and release separately.

## Your challenge

- [ ] Add an owning reservation with checked page-aligned subranges and native
  error reporting; release exactly the owned range without panicking.
- [ ] Keep platform calls injectable for range and failure tests.
- [ ] Allocate through a simple locked bump frontier, committing pages on demand.
- [ ] Publish correct managed bounds and write-barrier/card state for the pinned
  runtime; validate all barrier paths used by the managed fixtures.
- [ ] Preserve the suspension, required allocation-context callbacks, and root
  handoff contracts while keeping allocation on a correct slow path.
- [ ] Run the managed graph and runtime-root diagnostics against the new ranges.
- [ ] Preserve bounded exhaustion and reconcile reserved, committed, and owned
  bytes through partial initialization and failure.

## Checkpoint

Managed fixtures resume with the same observed graphs and declared root
coverage. Every allocation lies in an aligned committed owned range; limits and
injected reserve/commit failures terminate without memory corruption.

## Allowed shortcuts

- One reservation, one active range, and a global allocation lock are sufficient.
- Per-thread fast allocation and collection are not required.
- Allocation records may support diagnostics until byte walking is established.

## Known debt

The stopped heap cannot yet be reconstructed without allocation bookkeeping.
Owned storage alone does not make an incomplete trace safe for reclamation.

## What this unlocks

Mission 14 verifies object boundaries across the entire stopped allocated range.

## Hints

State the reservation owner's lifetime and thread-safety rules explicitly.
