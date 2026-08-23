# Study resources

The list is ordered for this project rather than by prestige. Read only what supports the current milestone, take short notes, and turn each concept into a test or experiment.

## Primary path: the MiniDump series

Kevin Gosse's series is the closest implementation guide to this project. Read the matching part immediately before the mission that needs it, then inspect the tagged source for details that the article omits.

1. [Part 1 — project setup and standalone GC loading](https://minidump.net/2025-28-01-writing-a-net-gc-in-c-part-1/)
2. [Part 2 — minimal allocating GC, allocation contexts, handles, and write barrier](https://minidump.net/writing-a-net-gc-in-c-part-2/)
3. [Part 3 — inspecting objects with the DAC](https://minidump.net/writing-a-net-gc-in-c-part-3/)
4. [Part 4 — heap walking, segments, and free objects](https://minidump.net/writing-a-net-gc-in-c-part-4/)
5. [Part 5 — decoding positive and negative GCDesc forms](https://minidump.net/writing-a-net-gc-in-c-part-5/)
6. [Part 6 — roots, iterative marking, and sweeping](https://minidump.net/writing-a-net-gc-in-c-part-6/)
7. [Part 7 — stable handle storage, weak/dependent handles, and fixed-point scanning](https://minidump.net/writing-a-net-gc-in-c-part-7/)
8. [Part 8 — interior pointers and brick tables](https://minidump.net/writing-a-net-gc-in-c-part-8/)
9. [Part 9 — frozen segments and reserving a contiguous managed address range](https://minidump.net/writing-a-net-gc-in-c-part-9/)
10. [Part 10 — finalization, f-reachable queues, resurrection, weak ordering, and sync blocks](https://minidump.net/writing-a-net-gc-in-c-part-10/)

Source: [kevingosse/ManagedDotnetGC](https://github.com/kevingosse/ManagedDotnetGC). Its branch/tag history mirrors the series. As of part 10 it is an educational Windows x64, .NET 10, stop-the-world, non-generational, non-compacting collector and still does not reuse reclaimed space. Use it as a behavioral and interface reference, not as code to transliterate line by line.

## .NET runtime sources and design documents

Read these as the source of truth for the pinned runtime version:

- [CoreCLR Garbage Collection Design (Book of the Runtime)](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
- [Standalone GC loader design](https://github.com/dotnet/runtime/blob/main/docs/design/features/standalone-gc-loading.md)
- [`gcinterface.h` — GC/EE contract](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/gcinterface.h)
- [`gcinterface.ee.h` — execution-engine side](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/gcinterface.ee.h)
- [`gcdesc.h` — object reference metadata](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/gcdesc.h)
- [`gc.cpp` — allocator and collector implementation](https://github.com/dotnet/runtime/blob/main/src/coreclr/gc/gc.cpp)
- [Workstation and Server GC documentation](https://learn.microsoft.com/dotnet/standard/garbage-collection/workstation-server-gc)
- [GC runtime configuration](https://learn.microsoft.com/dotnet/core/runtime-config/garbage-collector)

Always link to a commit, not `main`, in an implementation ADR. Interface layout can change between runtime releases.

Small/reference implementations:

- [kkokosa/UpsilonGC](https://github.com/kkokosa/UpsilonGC), especially the bump-pointer `ZeroGC` example.
- [CoreCLR sample GC directory](https://github.com/dotnet/runtime/tree/main/src/coreclr/gc/sample).

## Satori and low-latency direction

- [VSadov/Satori](https://github.com/VSadov/Satori)
- [Satori collector README and feature summary](https://github.com/VSadov/Satori/tree/main/src/coreclr/gc/satori)
- [Satori region implementation](https://github.com/VSadov/Satori/blob/main/src/coreclr/gc/satori/SatoriRegion.h)
- [.NET runtime discussion 115627](https://github.com/dotnet/runtime/discussions/115627)

Concepts to study rather than copy:

- Region-owned metadata.
- Thread-local Gen0 and escape tracking.
- Concurrent phases plus mutator assistance/pacing.
- Optional compaction driven by fragmentation.
- Low-latency mode disabling heap-size-sensitive blocking phases.
- Auto-tuning based on measured allocation and collection behavior.

Important caution: Satori changes more than a replaceable GC library. Its design is closely coupled to runtime barriers and helpers. A standalone DLL with a very different collector cannot assume those runtime changes are present.

## Books

### Read alongside implementation

1. **Pro .NET Memory Management, 2nd edition** — Konrad Kokosa, Christophe Nasarre, and Kevin Gosse. Continue the book already in progress. Prioritize object layout, generations/segments, roots, handles, finalization, pinning/POH, write barriers/card tables, background GC, regions, and diagnostics.
2. [**The Garbage Collection Handbook, 2nd edition**](https://www.routledge.com/The-Garbage-Collection-Handbook-The-Art-of-Automatic-Memory-Management/Jones-Hosking-Moss/p/book/9781032231785) — Richard Jones, Antony Hosking, and Eliot Moss. For this project, start with tracing, mark-sweep, allocation/free lists, generational collection, incremental/concurrent correctness, parallel collection, and real-time/bounded-latency chapters. Do not read all 600 pages before coding.
3. [**Rust Atomics and Locks**](https://marabos.nl/atomics/) — Mara Bos. Read chapters 1-4 before a concurrent data structure, chapter 6 for strong/weak ownership intuition, and chapter 7 before reasoning about x64 versus ARM64 ordering.

### Useful later

- **Systems Performance, 2nd edition** — Brendan Gregg. Use for experimental design, CPU/cache/VM investigation, and avoiding misleading benchmark conclusions.
- **Rust for Rustaceans** — Jon Gjengset. The chapters on unsafe code, APIs, concurrency, and FFI are useful when the project moves from the model to native integration.

`CLR via C#` is useful historical context but predates modern .NET GC regions, POH, DATAS, and recent runtime work. Prefer current runtime source when it disagrees with older books.

## Rust-specific material

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — unsafe contracts, uninitialized memory, aliasing, `Send`/`Sync`, and FFI.
- [Unsafe Code Guidelines reference](https://rust-lang.github.io/unsafe-code-guidelines/) — especially layout, aliasing vocabulary, and pointer provenance.
- [Strict provenance RFC 3559](https://rust-lang.github.io/rfcs/3559-rust-has-provenance.html) — why an address is not the whole meaning of a Rust pointer.
- [Miri](https://github.com/rust-lang/miri) — use on `gc-core`; it cannot validate most platform FFI.
- [Loom](https://github.com/tokio-rs/loom) — explore interleavings in custom atomic queues/freelists.
- [The Rust Fuzz Book](https://rust-fuzz.github.io/book/) — fuzz metadata decoders and heap walkers.
- [Rust as a Language for High Performance GC Implementation](https://www.mmtk.io/assets/pubs/rust-ismm-2016.pdf) — a directly relevant discussion of using Rust for collector implementation.

## MMTk: the best comparison architecture

- [What is MMTk?](https://docs.mmtk.io/tutorial/intro/what_is_mmtk.html)
- [MMTk tutorial: build GC plans from NoGC to generational copying](https://docs.mmtk.io/tutorial/prefix.html)
- [mmtk-core API](https://docs.mmtk.io/api/mmtk/)
- [mmtk-core source](https://github.com/mmtk/mmtk-core)

MMTk already separates runtime-neutral collector mechanisms from VM bindings in Rust. Do the tutorial after the baseline model, then compare its `Plan`, `Space`, `Policy`, object model, and VM binding boundaries with this repository. Reconsider using MMTk after v0.2, not halfway through the initial standalone-GC learning exercise.

## Papers and production collectors

Read abstracts and design sections first; only go deeper when an experiment needs the mechanism.

- [Immix: A Mark-Region Garbage Collector](https://www.steveblackburn.org/pubs/papers/immix-pldi-2008.pdf) — mark-region organization, line reuse, fragmentation, and opportunistic evacuation.
- [Low-Latency, High-Throughput Garbage Collection (LXR)](https://arxiv.org/abs/2210.17175) — modern regional low-latency trade-offs and tail-latency evaluation.
- [OpenJDK ZGC project](https://wiki.openjdk.org/display/zgc/Main) — concurrent moving collection and colored-pointer/load-barrier techniques.
- [OpenJDK Shenandoah project](https://wiki.openjdk.org/display/shenandoah/Main) — concurrent compaction and pause-time goals.
- [Unity incremental GC documentation](https://docs.unity3d.com/Manual/performance-incremental-garbage-collection.html) — game-loop motivation, incremental work, and the cost of write barriers.

These are advanced comparison points. None should expand the 2026 implementation scope into a concurrent moving collector.

## Diagnostics and benchmarking

- [PerfView](https://github.com/microsoft/perfview) for ETW/EventPipe analysis on Windows.
- [`dotnet-trace`](https://learn.microsoft.com/dotnet/core/diagnostics/dotnet-trace) for portable trace collection.
- [`dotnet-counters`](https://learn.microsoft.com/dotnet/core/diagnostics/dotnet-counters) for live sanity checks.
- [BenchmarkDotNet](https://benchmarkdotnet.org/) for managed microbenchmarks, with a custom runner for frame deadlines and GC pause events.
- [GCPerfSim](https://github.com/dotnet/performance/tree/main/src/benchmarks/gc) for standard GC stress patterns.

Benchmark warnings:

- Do not infer pause quality from average time.
- Separate execution-engine suspension from collector phase time.
- Treat warmup, JIT mode, CPU frequency, background load, and power plan as experimental controls.
- Report committed memory and working set as well as managed heap size.
- Test clustered deaths and scattered deaths; compaction and free-list policies react very differently.
- Keep Satori results optional and record the exact fork commit and runtime build.

## Suggested reading sequence by mission horizon

Read for the problem currently being observed, not for the final collector in
advance.

| Missions | Read first | Keep open while experimenting |
| --- | --- | --- |
| 00-02 | MiniDump 1; standalone loader design | pinned `gcinterface.h`; Rustonomicon FFI |
| 03-04 | MiniDump 2; Pro .NET Memory Management “Custom GC” | pinned `gcinterface.ee.h`; ZeroGC/UpsilonGC |
| 05-07 | GC Handbook tracing and mark-sweep introduction | Miri; a simple independent graph oracle |
| 08-12 | GC Handbook allocation, sweep, and free-space chapters | MMTk tutorial; unsafe guidelines |
| 13-14 | MiniDump 2 and 4; Pro .NET allocation and virtual-memory chapters | Windows VM documentation; pinned runtime allocator paths |
| 15-16 | MiniDump 3-5 | pinned object-layout sources, `gcdesc.h`, fixture diagnostics |
| 17-19 | MiniDump 6; CoreCLR GC design | pinned `gc.cpp`; suspension/root callbacks |
| Future only | MiniDump 7-10, then allocation/fragmentation and regional literature as needed | Satori, LXR, Unity incremental GC, benchmark tools |
