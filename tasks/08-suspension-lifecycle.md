# Mission 08 - Suspend and resume around a diagnostic operation

## Where you are

Managed graph fixtures have explicit expected relationships and a continuation
marker. Their native observation window is not established.

## The problem

Even a read-only traversal can observe inconsistent references while threads
mutate them. Suspension and restart need an independently tested owner.

## Observe first

Trace the pinned `IGCToCLR` suspend/restart contract and the fixture's transition
into native code. Identify which thread may request suspension without deadlock.

## Your challenge

- [ ] Define running, suspending, suspended, restarting, and failed states;
  reject re-entry and invalid transitions.
- [ ] After successful suspension, perform a bounded empty diagnostic operation
  and attempt restart exactly once on every exit path.
- [ ] Validate the fixture handoff and borrowed-reference lifetime across this
  window; do not retain dereferenceable object pointers after restart.
- [ ] Implement the allocation-context callbacks actually required by suspension
  with correct semantics, even while allocation remains slow.
- [ ] Inject failure after suspension and before restart; report restart failure.
- [ ] Keep managed allocation and FFI unwinding out of the suspended operation.
- [ ] Start with one mutator and then demonstrate repeated cycles with two
  bounded mutators, resolving required callback contracts explicitly.

## Checkpoint

Managed execution reaches its continuation marker after successful diagnostic
cycles. Every injected post-suspend failure attempts restart exactly once;
re-entry is rejected and restart failure is observable.

## Allowed shortcuts

- The suspended operation may be empty.
- No heap walk, object decoding, root scan, or allocation fast path is required.
- Native fault-injection tests may use an injectable execution-engine adapter.

## Known debt

Stopping threads establishes a window, not object layout knowledge or liveness.

## What this unlocks

Mission 09 reads a supported object only within this verified window.

## Hints

Preallocate diagnostic buffers or define bounded native-allocation failure
handling. Do not log through callbacks that can re-enter managed execution.
