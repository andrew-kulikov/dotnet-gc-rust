# Mission 12 - Diagnose reachability from runtime roots

## Where you are

Supported real objects can be traced safely from explicit fixture roots.

## The problem

CoreCLR supplies stack and runtime roots through callbacks with flags and
phase-limited lifetimes. A fixture root list cannot stand in for that contract.

## Observe first

Build fixtures retaining known objects through stack locals, statics,
thread-statics, and exception paths. Keep identity expectations from accidentally
adding strong references to objects intended to become unreachable.

## Your challenge

- [ ] Adapt the pinned `GcScanRoots` contract during the verified suspension.
- [ ] Record root source, flags, and validated object targets; borrowed slots must
  not escape suspension.
- [ ] Validate actual allocation ranges independently of ZeroGC's synthetic
  write-barrier bounds. Handle frozen/external memory only through an explicit
  supported rule, otherwise report incomplete analysis.
- [ ] Traverse supported objects with the tested reader and model semantics.
- [ ] Distinguish root-callback coverage from handle-table coverage. Account for
  each root source once and report any missing source or unsupported root flag.
- [ ] Return complete-for-declared-scope, incomplete, or corrupt outcomes. Never
  label the unmarked complement as collectible in this mission.
- [ ] Report phase timings and traversal counters, and reset state each cycle.
- [ ] Resume after unsupported cases, corruption, and injected work-budget errors.

## Checkpoint

Each claimed root source retains its known fixture graph over repeated runs.
An unsupported flag, shape, or root source yields an explicit incomplete result;
all post-suspend paths obey the restart guarantee. No memory is reclaimed.

## Allowed shortcuts

- Diagnostic snapshots and slow validation are allowed.
- The registry may enumerate candidate allocations and validate addresses; it
  must not supply liveness or imply root completeness.
- Unsupported interior, pinned, weak, or dependent semantics may abort tracing.

## Known debt

Successful fixture coverage is not proof of complete process liveness. Heap
walking, runtime handles, and a strict reclamation gate remain necessary.

## What this unlocks

Mission 13 establishes collector-owned storage while these runtime diagnostics
serve as an end-to-end regression check.

## Hints

Keep a trace-completeness result separate from the reachable set.
