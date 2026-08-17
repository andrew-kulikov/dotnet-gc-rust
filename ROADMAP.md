# Project roadmap

Planning date: 2026-08-17  
Baseline target: 2026-11-29  
Frame-aware experiment target: 2026-12-20  
Buffer and stretch work: 2026-12-21 through 2026-12-31

This schedule assumes roughly 6-10 focused hours per week. Treat dates as review points, not reasons to merge unsafe or untested code. If time becomes tight, preserve the baseline and cut stretch work in the order listed under "Scope cuts."

## Working method

Use one small pull request per observable capability. Each pull request should include:

- A short design note or updated invariant when memory layout changes.
- Tests that fail before the change and pass after it.
- A runnable demonstration or benchmark command.
- No unrelated refactoring.
- A note describing newly introduced `unsafe` code and how it was checked.

Keep a weekly engineering log in `docs/weekly/` with four headings: learned, built, measured, next. This turns the repository into a useful public learning record without exposing private notes.

The milestone plan is decomposed into small, sequential exercises under `tasks/`. Complete each exercise on its own branch and use `tasks/REVIEW_GUIDE.md` when asking an agent to verify it.

## Milestone 0 — Public skeleton and ABI risk spike

**Dates:** August 17-23  
**Outcome:** prove that the chosen toolchain can build a library that CoreCLR attempts to load.

Tasks:

1. Install a current stable Rust toolchain with `rustup`; record the pinned version in `rust-toolchain.toml`.
2. Pin one installed .NET 10 SDK/runtime version with `global.json` and document how to obtain it.
3. Confirm the required MSVC C++ Build Tools and Windows SDK are available.
4. Create the Cargo workspace and the first managed sample.
5. Export `GC_VersionInfo` and `GC_Initialize`; initially return failure deliberately and verify that CoreCLR reaches both exports.
6. Write ADR-0001 comparing:
   - a thin C++ ABI shim plus a stable C ABI into Rust (recommended), and
   - hand-built MSVC C++ vtables in pure Rust (rejected unless a contained experiment proves it maintainable).
7. Add Windows CI for the loader smoke test and cross-platform CI for `gc-core`.
8. Add `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, code-format checks, and dependency/license auditing before making the repository public.

Exit criteria:

- CI builds the native library and managed samples from a clean checkout.
- A test proves CoreCLR calls the exported version and initialization functions.
- No runtime binaries or machine-specific paths are tracked.

Rust concepts to practice: workspaces, `cdylib`, Rust 2024 unsafe attributes such as `#[unsafe(no_mangle)]`, `extern "C"`, `#[repr(C)]`, build scripts, error boundaries, and panic containment across FFI.

## Milestone 1 — Runtime-independent heap model

**Dates:** August 24-September 6  
**Outcome:** a pure-Rust object graph and heap model that can be tested without CoreCLR.

Tasks:

1. Define strongly typed `ObjectId`, `RegionId`, `HandleId`, byte sizes, and aligned offsets; avoid passing bare `usize` values through safe APIs.
2. Implement a contiguous arena split into fixed-size regions.
3. Add aligned bump allocation and explicit object headers in a byte buffer.
4. Define an object-layout descriptor used by the test VM to enumerate reference slots.
5. Define `RootSet`, `Trace`, `Allocator`, and `CollectorPolicy` traits only where they allow real substitution in tests.
6. Use typestate or a private phase enum so allocation cannot occur through safe APIs while the model is stopped for collection.
7. Generate random object graphs with property tests.

Exit criteria:

- Allocations obey alignment and never overlap.
- Walking a region reconstructs every object boundary.
- Invalid offsets, oversized objects, and arithmetic overflow fail cleanly.
- The FFI-free crate passes Miri.

Rust concepts to practice: newtypes, lifetimes, generics, traits, const generics where useful, `NonNull`, `MaybeUninit`, privacy as an unsafe boundary, and checked arithmetic.

## Milestone 2 — Baseline mark, sweep, and reuse in the model

**Dates:** September 7-20  
**Outcome:** a correct non-moving collector in the pure-Rust harness.

Tasks:

1. Implement iterative DFS marking with an explicit mark stack.
2. Keep mark bits in side metadata rather than relying on Rust object values.
3. Sweep unmarked objects into free objects so every region remains walkable.
4. Coalesce adjacent free objects.
5. Add segregated free lists and use them before bump allocation.
6. Add strong, weak, and dependent handles to the model. Resolve dependent handles to a fixed point.
7. Track allocated, live, reclaimed, free, and fragmented bytes.
8. Add differential tests against a simple reference collector implemented with ordinary Rust collections.

Exit criteria:

- Every reachable object survives and every unreachable object becomes reusable.
- Randomized collection sequences preserve graph and free-list invariants.
- Cycles, deep graphs, duplicate roots, weak handles, and dependent-handle chains are covered.
- A benchmark shows allocation, collection, and fragmentation metrics.

Rust concepts to practice: iterators, explicit stacks, bitmaps, intrusive data structures, safe wrappers over raw storage, and property-based testing.

## Milestone 3 — Native memory and a leak-only standalone GC

**Dates:** September 21-October 4  
**Outcome:** a .NET console application runs to completion using Rust-owned allocation segments, without collection yet.

Tasks:

1. Add a virtual-memory abstraction with reserve, commit, decommit, and release operations.
2. Reserve one large 64-bit address range and commit segments on demand.
3. Wrap `IGCToCLR`, `IGCHeap`, `IGCHandleManager`, and `IGCHandleStore` in the C++ shim; expose only narrow C functions to Rust.
4. Implement thread allocation contexts and a bump-allocation fast path.
5. Implement a growable, stable-address handle store.
6. Initialize a valid card table and write-barrier bounds; do not permanently neutralize the barrier because the regional experiment will need it.
7. Implement required stubs with explicit logging and fail-fast behavior for unsupported calls.
8. Run a managed smoke app that allocates small, large, pinned, and finalizable objects but intentionally never frees them.

Exit criteria:

- The managed smoke app exits normally under the custom GC.
- Allocations stay inside the reserved range and committed memory accounting is correct.
- Unsupported server GC and unsupported architecture configurations fail with a useful message.
- No Rust panic or C++ exception can unwind across the FFI boundary.

Rust concepts to practice: platform FFI, RAII guards for virtual memory, `Send`/`Sync` decisions, atomics, `UnsafeCell`, and ABI-safe error handling.

## Milestone 4 — CoreCLR object layout and heap walking

**Dates:** October 5-18  
**Outcome:** reliably enumerate allocated managed objects and all outgoing references.

Tasks:

1. Document the exact x64 managed object header, method-table pointer, array length, alignment, and free-object layout for the pinned runtime.
2. Port object-size computation with tests for classes, strings, arrays, boxed value types, and large objects.
3. Fill abandoned allocation-context tails with valid free objects.
4. Implement forward heap walking for every committed segment.
5. Decode positive and negative `GCDesc` encodings.
6. Build a separate managed fixture for inheritance, nested structs, reference arrays, and arrays of value types containing references.
7. Use DAC only as an optional diagnostic aid; do not make collection correctness depend on DAC.

Exit criteria:

- Heap walking reaches the exact allocation frontier without guessing.
- Reference enumeration matches the managed fixtures.
- Corrupt or impossible metadata is detected before an unbounded walk.

Rust concepts to practice: bit fields, pointer arithmetic, layout assertions, zero-copy views, callback adapters, and carefully scoped `unsafe` modules.

## Milestone 5 — Stop-the-world marking from real roots

**Dates:** October 19-November 1  
**Outcome:** collect dead managed objects in CoreCLR without reusing their memory yet.

Tasks:

1. Suspend and restart the execution engine with an RAII guard that always attempts restart on error.
2. Enumerate and fix active allocation contexts before walking the heap.
3. Scan stack and thread-static roots through `GcScanRoots`.
4. Mark transitively with an iterative work list.
5. Sweep dead objects into free objects; clear memory in debug builds to expose missed roots.
6. Unmark survivors and verify the heap after every stress collection.
7. Add forced-GC tests for cycles, deep graphs, multiple threads, exceptions, and rapidly dying objects.

Exit criteria:

- The baseline object-graph suite survives repeated forced collections.
- Known-dead objects are observed as dead and the heap remains walkable.
- Execution-engine restart occurs on every tested error path.

Rust concepts to practice: state guards, callback trampolines, work queues, thread-safety boundaries, and failure atomicity.

## Milestone 6 — Runtime correctness features

**Dates:** November 2-15  
**Outcome:** cover the root categories and special heap areas needed by ordinary .NET code.

Implement in this order, keeping each item behind a feature gate until its tests pass:

1. Strong and pinned handles.
2. Short and long weak handles.
3. Dependent handles with fixed-point rescanning.
4. Interior pointers using a region-local brick/index table.
5. Frozen-segment registration and an O(1) managed-range check so frozen objects are never marked in-place.
6. Finalization queue, normal and critical f-reachable queues, suppression, re-registration, resurrection tests, and weak-reference ordering.
7. Sync-block weak-pointer scanning and eager finalization required by the pinned runtime.

Exit criteria:

- Each feature has a stock-GC differential test and a custom-GC stress test.
- `Span<T>`, `ConditionalWeakTable`, weak references, finalizers, string literals, pinned buffers, and contended locks run without corruption.
- Unsupported unloadable assembly contexts are detected, documented, and either safely rejected or explicitly added to the next milestone.

Rust concepts to practice: tagged freelists, atomics and the ABA problem, stable addresses, fixed-point algorithms, and synchronization contracts.

## Milestone 7 — Reclamation, stabilization, and v0.1

**Dates:** November 16-29  
**Outcome:** a publishable baseline that reuses memory.

Tasks:

1. Connect swept free objects to the segregated free lists implemented in the model.
2. Split large free objects on allocation and coalesce adjacent free objects during sweep.
3. Rebuild or update the interior-pointer index whenever object boundaries change.
4. Decommit wholly free regions after a delay; keep a small reuse cache to avoid commit/decommit thrashing.
5. Add allocation-failure and low-memory tests.
6. Run long randomized soak tests and native sanitizers.
7. Document the supported matrix, known limitations, build steps, and debugging workflow.
8. Tag `v0.1.0` only after reproducing the full build and tests from a clean public checkout.

Exit criteria:

- A steady-state allocate/drop workload reaches a bounded memory plateau.
- Free-list, brick-table, and region invariants survive the soak test.
- The public README accurately describes limitations; no "pauseless," "real-time," or production claim is made.

## Milestone 8 — Frame-aware regional policy

**Dates:** November 30-December 13  
**Outcome:** an experimental policy that can collect a bounded selection of regions at an explicit frame safe point.

Design hypothesis:

- Divide the heap into small regions with per-region occupancy, dirty-card, and estimated-reclaim statistics.
- At a managed safe point, select one or a few victim regions whose estimated work fits a requested microsecond budget.
- During the stop, scan roots and remembered-set/card entries that may point into the victim set, then trace only objects needed to determine liveness inside that set.
- Sweep the victim regions, leaving other regions untouched.
- Fall back to a full baseline collection when memory pressure, remembered-set cost, or fragmentation makes a partial collection unproductive.

Tasks:

1. First implement the policy in `gc-core` and compare it with full-heap collection on generated graphs.
2. Add per-region live-byte estimates, allocation age, free-list quality, and dirty-card density.
3. Create a policy interface with `Throughput`, `Interactive`, and `FrameBudget` modes.
4. Add a managed control API such as `CollectAtSafePoint(budget_microseconds)` for experiments. Clearly document that the budget is a target, not a deadline guarantee.
5. Measure thread-suspension/root-scan time separately from region mark/sweep time; it is a fixed floor the policy cannot eliminate.
6. Add an emergency watermark that permits a full collection rather than risking out-of-memory failure.
7. Test pathological cross-region graphs and high dirty-card density; automatically abandon partial collection when it would do more work than a full collection.

Exit criteria:

- Partial collections never reclaim an object reachable from roots or non-victim regions.
- The controller's estimates correlate with measured work and choose fewer regions under a tighter budget.
- Emergency fallback is observable and tested.

Why this is game-oriented: it makes pause shaping and safe-point placement explicit, accepting additional memory, barriers, fragmentation, and occasional fallback pauses. It is inspired by the regional and low-latency direction of Satori, but it is not Satori's concurrent or thread-local algorithm.

## Milestone 9 — Game-shaped benchmark report and v0.2

**Dates:** December 14-20  
**Outcome:** a reproducible report that decides whether the experiment worked.

Workloads:

1. **Frame churn:** 60 Hz and 120 Hz loops with short-lived per-frame objects.
2. **Asset lifetime:** a large mostly-stable graph plus small transient updates.
3. **Scene transition:** burst allocation followed by clustered death.
4. **Fragmentation:** random-sized objects with scattered deaths.
5. **Cross-region worst case:** frequent old-to-young and region-to-region pointer updates.
6. **Runtime features:** pinned buffers, spans/interior pointers, weak caches, dependent handles, and finalizers.

Comparisons:

- Custom full-heap baseline.
- Custom frame-budget policy with at least three budgets.
- Stock Workstation GC.
- Stock Server GC and its relevant latency modes.
- Satori only as an optional external comparison if a reproducible compatible build is available; it must not block the release.

Report:

- Allocation throughput and total elapsed time.
- GC CPU time and total pause time.
- Pause `p50`, `p95`, `p99`, `p99.9`, and maximum.
- Missed 60 Hz and 120 Hz frame deadlines.
- Peak and post-workload committed memory.
- Reclaimed bytes per millisecond and fragmentation.
- Hardware, OS class, power plan, runtime commit, warmup, trial count, and confidence intervals.

Exit criteria:

- Benchmark scripts and raw machine-readable results are committed.
- The report states where the policy wins, loses, and falls back.
- `v0.2.0` is tagged even if the result is negative, provided correctness holds and the analysis is honest.

## Milestone 10 — Buffer and stretch experiments

**Dates:** December 21-31

Use this time first for defects, documentation, and reproducibility. Attempt one stretch item only if v0.2 is already complete:

1. Prototype thread-local allocation regions and escape tracking in the pure-Rust model.
2. Prototype concurrent sweeping, which is safer and simpler than concurrent marking.
3. Add a compacting collection for one unpinned region.
4. Add Linux x64 platform memory support without claiming CoreCLR integration support.
5. Evaluate whether an MMTk plan plus a .NET binding would be a better long-term architecture.

Do not start concurrent marking in 2026 unless a written barrier protocol, snapshot invariant, and race-testing strategy already exist.

## Scope cuts if behind schedule

Cut from the bottom upward:

1. Satori comparison and all stretch experiments.
2. Adaptive auto-tuning; keep manually selected frame budgets.
3. Decommit trimming; keep region reuse.
4. Critical/eager finalization polish if ordinary finalization is correct and unsupported cases fail safely.
5. Regional policy integration with CoreCLR; retain the complete pure-Rust experiment and publish the result.

Never cut:

- Heap and free-list verification.
- Tests for roots, handles, interior pointers, frozen objects, and finalization paths that are claimed as supported.
- FFI panic containment and execution-engine restart guards.
- Public disclosure of limitations.

## Release checklist

- Clean clone builds without local absolute paths.
- All dependencies and adapted sources have compatible licenses and attribution.
- No SDK/runtime binaries, dumps, traces, secrets, usernames, or hostnames are tracked.
- `cargo fmt`, `clippy`, unit tests, property tests, managed differential tests, stress tests, and relevant sanitizers pass.
- Unsafe-code inventory and invariants are current.
- Runtime version and ABI header commit are pinned.
- Benchmarks include raw data and exact reproduction commands.
- README claims match the tests and measurements.
