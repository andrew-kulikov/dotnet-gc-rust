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

First prove the loader boundary, then make initialization succeed, then run the
smallest managed program with an intentionally poor and bounded ZeroGC. This
early implementation may allocate every object separately and leak until a
configured limit.

The gate is complete when:

- stock and custom runs are separately reproducible;
- `LoaderSmoke` reaches `Main` and exits normally under the custom GC;
- unsupported operations fail by name rather than returning fake success;
- the ZeroGC workload and counters make its limitations visible.

### Gate B - collector semantics in a replaceable model

Learn reachability, reclamation ordering, strong/weak/dependent handles, byte
walking, and free-space reuse without CoreCLR or native pointers. Begin with
ordinary Rust collections, then replace them with a byte heap only when object
boundaries become the problem. Add typed byte arithmetic only when multiple
coordinate systems make `usize` ambiguous.

The gate is complete when:

- generated graph results match an independent oracle;
- a byte walker reconstructs objects without allocator bookkeeping;
- dead records remain walkable and reusable;
- fragmentation workloads justify or reject free-space indexes with data;
- the FFI-free crate passes normal tests and Miri-compatible tests.

### Gate C - collector-owned managed allocation

Replace ZeroGC's scattered allocations with a reserved, lazily committed address
range. Start with a slow locked allocator, observe contention, then refactor to
thread allocation contexts. Derive real object sizes from pinned runtime source
and decode outgoing reference metadata incrementally from managed fixtures.

The gate is complete when:

- managed allocations stay inside owned committed ranges;
- allocation contexts never overlap and all tails are accounted for;
- supported objects walk exactly to known frontiers;
- outgoing references match independent fixture expectations;
- corrupt metadata fails in bounded time before unsafe traversal.

### Gate D - first real stop-the-world collection

Establish suspension and restart as an independently tested lifecycle. Then add
runtime roots, real marking, and finally sweep dead objects into valid walkable
free records. Do not reuse those real free records in the same mission.

The gate is complete when:

- every tested post-suspend exit attempts restart exactly once;
- known roots keep known graphs live across repeated mark cycles;
- known-dead objects become valid free records;
- the heap verifies and byte accounting reconciles after every sweep;
- stress behavior does not depend on diagnostic logging.

## Future direction - intentionally unscheduled

Create the next mission batch only after Gate D passes. Use actual failures and
measurements to choose the order among:

- strong and pinned runtime handles;
- short/long weak and dependent runtime handles;
- interior pointers and object-start indexing;
- frozen segments;
- finalization, resurrection, critical finalizers, and sync-block weak state;
- real-heap split/coalesce/reuse and bounded-memory stabilization;
- low-memory, failure-injection, and long soak validation;
- regional victim selection and partial-collection correctness;
- remembered sets and write-barrier integration;
- managed frame-safe-point control and emergency fallback;
- reproducible pause, throughput, memory, and fragmentation evaluation.

The regional experiment remains a hypothesis: trading memory and bookkeeping
for tighter tail pauses may or may not help the defined workload. A negative
measured result is acceptable; a collector with an unverified root or heap
invariant is not.

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
