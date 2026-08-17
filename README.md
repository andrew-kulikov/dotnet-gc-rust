# dotnet-gc-rust

An educational .NET garbage collector prototype written primarily in Rust.

The project has two goals:

1. Learn how CoreCLR allocation, object layout, root discovery, handles, finalization, and the standalone GC contract work.
2. Explore a game-oriented policy that trades memory and some throughput for shorter, more predictable stop-the-world pauses.

This is a learning and research project. It is not intended for production use, security-sensitive workloads, or compatibility with arbitrary .NET versions.

## Planned scope

The first complete release will target:

- Windows x64 only.
- One pinned .NET 10 SDK/runtime version.
- A standalone GC loaded by CoreCLR.
- A non-generational, non-moving, stop-the-world mark-and-sweep baseline.
- Reusable free space through coalescing and segregated free lists.
- Strong, pinned, weak, and dependent handles.
- Interior pointers, frozen segments, and finalization.
- A managed behavioral test application plus a runtime-independent Rust test harness.

The experimental second release will add a regional, frame-aware collection policy. It will cap the amount of heap work selected for one collection and allow a managed game loop to request collection at a safe point between frames. The intended trade-off is higher memory use and bookkeeping cost in exchange for a tighter pause distribution.

See [ROADMAP.md](ROADMAP.md) for the implementation schedule, [docs/DESIGN.md](docs/DESIGN.md) for the proposed architecture, and [docs/RESOURCES.md](docs/RESOURCES.md) for the reading list.

## Non-goals for 2026

- Production readiness or formal real-time guarantees.
- Full compatibility with every CoreCLR GC feature or runtime version.
- Server GC, 32-bit targets, Mono, NativeAOT as the host runtime, or cross-platform integration.
- A fully concurrent or moving collector.
- Reproducing Satori. Satori is a mature runtime fork with custom barrier and runtime integration; this project borrows bounded-work and regional ideas at a much smaller scale.

## Proposed repository layout

```text
crates/
  gc-core/          Runtime-independent heap, tracer, policies, and model tests
  gc-platform/      Virtual-memory and timing abstraction; Windows implementation
  gc-runtime/       CoreCLR object layout, GCDesc decoding, roots, and handles
  gc-ffi/           Stable C ABI exposed by the Rust library
native-shim/        Thin C++ implementation of the versioned CoreCLR GC interfaces
managed-harness/    C# behavioral tests and frame-shaped workloads
benchmarks/         Reproducible workloads, runners, and result schemas
docs/               Design notes, ADRs, safety invariants, and research notes
```

The C++ shim should contain no collection policy. It exists only because CoreCLR's GC/EE interfaces are C++ virtual interfaces whose ABI is not a stable Rust FFI boundary. All allocator and collector behavior should remain in Rust behind a small C ABI.

## Quality bar

Every milestone should leave the repository in a runnable state. The project will use:

- Unit, property, and model-based tests for the runtime-independent collector.
- Differential behavioral tests that run against both the stock .NET GC and this GC.
- Stress tests with forced collections, graph mutation, weak references, finalizers, pinning, and interior pointers.
- Miri for the FFI-free core, plus platform sanitizers and page-guard tests for native integration.
- Measured pause distributions (`p50`, `p95`, `p99`, `p99.9`, and maximum), not only average pause time.
- Safety comments on every `unsafe` block and written invariants for every raw-pointer-owning type.

## Public-repository rules

- Never commit runtime binaries, SDK files, dumps, traces, benchmark machine identifiers, absolute local paths, credentials, or environment files.
- Download or locate redistributable runtime components during local setup or CI; do not vendor Microsoft binaries.
- Record the exact runtime commit/version used by the ABI shim.
- Preserve upstream license notices for adapted code and cite the source in the relevant file or design note.
- Publish benchmark scripts, raw numeric results, machine-class information, and methodology, but remove usernames, hostnames, and unrelated process data.

## Definition of success

The baseline is complete when it can repeatedly run the managed conformance workload, reclaim and reuse dead objects, and pass stress tests without heap corruption.

The frame-aware experiment is successful only if a documented workload demonstrates a materially tighter tail-pause distribution than the baseline while clearly reporting its memory and throughput cost. A good initial research target is:

- `p99` GC pause at or below 2 ms in the defined 120 Hz synthetic workload.
- No GC pause above 5 ms during the measured steady-state interval.
- At least 70% of baseline allocation throughput.
- Peak committed memory no more than 1.75 times the baseline.

These numbers are hypotheses for one controlled benchmark, not product guarantees. If the experiment misses them, the negative result and the reason are still a valid project outcome.

