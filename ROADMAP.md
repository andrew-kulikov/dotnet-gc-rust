# Project direction

This file describes desired outcomes and boundaries, not a predetermined
implementation schedule. The executable learning path lives in `tasks/` and is
expected to change when experiments disprove an assumption.

## Planning rules

1. Prefer a small end-to-end result over completing one architectural layer.
2. Start each mission by reproducing a concrete limitation.
3. Permit intentionally slow or disposable implementations when their behavior
   is correct, bounded, and documented.
4. Introduce abstractions only after the current implementation exposes the
   problem they solve.
5. Preserve observable checkpoints while allowing large internal rewrites.
6. Detail only the current learning horizon. Future work remains an outcome, not
   a checklist, until the preceding gate passes.
7. Correctness, heap verification, FFI containment, and honest unsupported
   diagnostics are never optional shortcuts.

## Supported learning target

- Windows x64.
- One pinned .NET 10 SDK/runtime and matching `dotnet/runtime` source revision.
- A standalone GC loaded by CoreCLR through a thin C++ ABI adapter.
- Collector state and policy owned by Rust behind a narrow C ABI.
- A non-moving, stop-the-world baseline before latency experiments.
- Educational and research use only.

Server GC, 32-bit targets, Mono, NativeAOT hosting, production readiness, and
formal real-time guarantees are outside the current target.

## Current outcome gates

### Gate A - observable standalone-GC startup

Missions 00-04 establish the loader boundary, bounded managed execution,
explicit unsupported behavior, and reconciled allocation diagnostics.

### Gate B - real fixture tracing

Missions 05-10 connect the model to managed execution in small verifiable steps:

- graph reachability and model reclamation match independent expectations;
- C# fixtures provide known relationships and explicit roots;
- suspension and restart have an independently tested lifecycle;
- a supported real object's size and references come from pinned metadata;
- snapshots of real fixture links match the graph oracle;
- every supported diagnostic run resumes managed execution.

Fixture-relative reachability never authorizes runtime reclamation.

### Gate C - runtime coverage and heap inventory

Missions 11-14 extend real shape decoding, observe runtime roots, introduce owned
storage, and establish a full stopped-heap walk.

The gate is complete when:

- every claimed shape and root source has an independent managed fixture;
- unsupported cases produce incomplete analysis without liveness claims;
- managed allocation remains inside owned committed ranges;
- heap bytes reconstruct the complete supported inventory without depending on
  the allocation registry;
- runtime graph diagnostics survive the storage transition.

### Gate D - complete marking and first real sweep

Missions 15-18 specify model handle ordering, implement the required native
handle contracts, and make trace completeness a prerequisite for mutation.

The gate is complete when:

- every liveness source required by the declared workload participates;
- unknown roots, flags, layouts, or handles prevent sweep authorization;
- complete marking, weak clearing, and sweep run in one suspended window;
- known-live objects survive and known-dead ranges become valid free records;
- heap verification and byte accounting succeed before normal execution resumes;
- recoverable failures preserve exactly one restart attempt; unrecoverable heap
  corruption follows an explicit terminal policy instead of normal resumption.

This is a supported experimental workload, not general runtime completeness.
Native freed ranges are not yet reused.

### Gate E - thread allocation contexts

Mission 19 introduces a fast allocation path while preserving the existing
walk/mark/sweep invariants. Active tails are closed into valid records during
suspension, and every context byte is accounted for.

## Focused model laboratories

The laboratories listed in [tasks/README.md](tasks/README.md) explore byte
records, typed region arithmetic, reuse, fragmentation, and dependent handles
when a concrete problem requires them. They provide independent oracles for
native work without blocking early real-object tracing.

## Future direction - intentionally unscheduled

Create additional missions from observed runtime failures and measurements:

- broader strong, pinned, weak, and dependent handle coverage;
- interior pointers, frozen segments, and object-start indexing;
- finalization, resurrection, critical finalizers, and sync-block weak state;
- native split/coalesce/reuse and bounded-memory stabilization;
- low-memory, failure-injection, and long soak validation;
- regional victim selection and partial-collection correctness;
- remembered sets and write-barrier integration;
- managed frame-safe-point control and emergency fallback;
- reproducible pause, throughput, memory, and fragmentation evaluation.

A missing runtime capability blocks collection whenever the workload encounters
it. Scheduling its general support later never permits an incomplete sweep.

The regional experiment remains a hypothesis: trading memory and bookkeeping
for tighter tail pauses may or may not help the defined workload. A negative
measured result is acceptable; unverified root or heap invariants are not.

## Decisions deliberately postponed

Do not decide these globally before a mission creates evidence:

- final region size and count;
- exact free-list bucket layout;
- which generic traits are useful substitution boundaries;
- lock-free handle or free-list structures;
- partial-collection remembered-set precision;
- decommit caching and trimming policy;
- frame-budget controller tuning.

Record a decision as an ADR only when it constrains multiple later missions or
an external contract. Local implementations are allowed to be replaced without
an ADR.

## Persistent quality bar

- The pinned runtime/source relationship is visible and checked.
- No Rust panic or C++ exception unwinds across FFI.
- Unsupported behavior fails explicitly before corrupting state.
- Every raw-memory owner documents range, alignment, lifetime, mutation phase,
  invalidation, and thread-safety rules.
- Every unsafe operation has a local `SAFETY` explanation tied to enforced
  preconditions.
- Model algorithms use independent differential oracles where practical.
- Managed behavior is compared under stock and custom GC.
- Claims follow tests and measurements; they do not anticipate future missions.
