# Mission 08: suspension lifecycle

[Mission 08](../../tasks/08-suspension-lifecycle.md) establishes a safe interval
for native diagnostics: stop managed execution, perform bounded work, and
attempt restart exactly once after each successful suspension.

The diagnostic operation is initially empty. Object decoding, root scanning,
tracing, reclamation, and allocation fast paths belong to later missions.

## Chapters

1. [The suspension contract](01-observe-first.md) explains the managed baseline,
   the pinned CoreCLR interface, thread ownership, and reference lifetimes.
2. [The first empty cycle](02-empty-cycle.md) describes the implementation,
   required callbacks, validation commands, and remaining work.

## Current state

The `chain` fixture completes one empty suspension cycle under ZeroGC, makes
one restart attempt, and returns to managed execution. Native tests cover
lifecycle failures and completion-event behavior. The existing bounded
`LoaderSmoke threads` scenario also passes, exercising thread-exit cleanup.

Mission 08 is still incomplete. Repeated suspension with a second active managed
mutator and handle handoff remain unvalidated.

## Plan

| Step | Outcome | State |
| --- | --- | --- |
| 1 | Establish the managed baseline and CoreCLR contract | Documented in chapter 1 |
| 2 | Give one owner responsibility for the lifecycle and restart | Implemented and unit-tested |
| 3 | Connect the managed entry, C++ shim, and Rust cycle | One empty cycle validated |
| 4 | Implement required state, waiting, and allocation-context callbacks | Implemented; managed waiting during repeated cycles remains unvalidated |
| 5 | Validate handle handoff and borrowed-reference lifetimes | Pending |
| 6 | Validate failure cleanup and restart-failure reporting | Native tests pass; full checkpoint pending |
| 7 | Demonstrate repeated cycles with two bounded mutators | Next experiment |
| 8 | Provide a reproducible full Mission 08 checkpoint | Pending |

The next experiment repeats the empty cycle while a second bounded managed
mutator runs. It must establish worker progress before and after suspension,
successful managed continuation, and clean worker exit.

See the [mission workflow](../../tasks/README.md),
[review guide](../../tasks/REVIEW_GUIDE.md), and
[fixture handoff contract](../../samples/GraphFixtures/README.md#pinned-runtime-handoff-contract)
for the wider project requirements.
