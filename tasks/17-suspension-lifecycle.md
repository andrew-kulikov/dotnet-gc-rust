# Mission 17 - Always restart managed execution

## Where you are

The collector can walk supported objects and enumerate their outgoing
references, but only diagnostics run while managed threads may still mutate the
heap.

## The problem

A real collection requires an execution-engine suspension. Any error after a
successful suspend must still attempt restart exactly once. Before scanning
roots or marking objects, prove the lifecycle independently of heap mutation.

## Observe first

Trace the pinned `IGCToCLR` suspend/restart contract and identify every callback
or diagnostic that is safe during suspension. Add injectable failures
immediately after suspend and at several later checkpoints.

## Your challenge

- [ ] Represent running, suspending, suspended, restarting, and failed states so
  invalid transitions and collection re-entry are rejected.
- [ ] Wrap successful suspension in one owner responsible for exactly one restart
  attempt on every exit path.
- [ ] Retire or close active allocation contexts before a diagnostic heap walk.
- [ ] Perform a read-only full heap verification while suspended, then restart
  and continue managed execution.
- [ ] Inject failures after suspension, context retirement, and heap verification
  and make restart attempts observable in native tests.
- [ ] Ensure suspension-path diagnostics do not allocate managed objects or
  unwind through ABI boundaries.

## Checkpoint

A multithreaded managed workload survives repeated suspend/verify/restart cycles.
Every injected post-suspend failure records exactly one restart attempt, and
double suspend/restart or re-entrant collection is rejected.

## Allowed shortcuts

- Do not enumerate roots or mark objects.
- The stopped phase may be slow and heavily verified.

## Known debt

A safe stop alone cannot determine liveness. Retired contexts and object walking
must now feed a real root-driven mark phase.

## What this unlocks

Mission 18 can adapt runtime root callbacks to the already tested model tracer
without simultaneously debugging suspension ownership.

## Hints

Test the observable restart action, not just the existence of a destructor. Plan
for restart failure to be reportable even though recovery may be impossible.
