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
3. [Repeated cycles](03-repeated-cycles.md) records the second-mutator experiment.
4. [Handle handoff and checkpoint](04-handoff-checkpoint.md) records the completed
   mission, failure checks, reproduction commands, and remaining limitations.
5. [Dependent handles](05-dependent-handles.md) removes the formatting workaround
   with tested ZeroGC handle operations.

## Current state

Mission 08 is complete for the pinned Windows x64 workstation runtime. All six
fixtures pass normal-handle handoff with one mutator, then eight cycles with two
bounded mutators. Cycle 4 injects an operation error after suspension; every cycle
attempts restart once, worker progress resumes, and managed continuation succeeds.
Native tests cover re-entry rejection, operation errors/unwinding, restart
failure, completion waiting, and allocation-context cleanup. No object memory is
decoded and no collection is performed.

## Plan

| Step | Outcome | State |
| --- | --- | --- |
| 1 | Establish the managed baseline and CoreCLR contract | Documented in chapter 1 |
| 2 | Give one owner responsibility for the lifecycle and restart | Implemented and unit-tested |
| 3 | Connect the managed entry, C++ shim, and Rust cycle | Empty cycle validated |
| 4 | Implement required state, waiting, and allocation-context callbacks | Native tests and repeated managed cycles pass; specific EE wait-branch coverage is not claimed |
| 5 | Validate handle handoff and borrowed-reference lifetimes | One owned normal handle; only copied status leaves the stopped operation |
| 6 | Validate failure cleanup and restart-failure reporting | Runtime operation-error injection and native restart-error tests pass |
| 7 | Demonstrate repeated cycles with two bounded mutators | Eight cycles per fixture; progress and clean exit checked |
| 8 | Provide a reproducible full Mission 08 checkpoint | Commands and observed results in chapter 4; added to CI |

Next action: Mission 09 may inspect one supported object's fields inside this
window. It must establish object layout separately before reading those fields.

See the [mission workflow](../../tasks/README.md),
[review guide](../../tasks/REVIEW_GUIDE.md), and
[fixture handoff contract](../../samples/GraphFixtures/README.md#pinned-runtime-handoff-contract)
for the wider project requirements.
