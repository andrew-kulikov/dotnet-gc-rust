# Mission 03 - Run Hello World with the worst correct GC

## Where you are

CoreCLR accepts the interface shell, then stops at a named unsupported operation.
No managed allocation succeeds under the custom GC.

## The problem

The project needs an early end-to-end success before it spends weeks designing a
heap. The fastest honest implementation is a ZeroGC: allocate valid managed
objects and never reclaim them. It will be slow and wasteful, but it will reveal
the real minimum contract required to reach managed `Main`.

## Observe first

Use mission 02's startup trace to identify the calls made before the first
allocation and during the first object allocation. Compare that trace with the
pinned interface definitions. Add one diagnostic counter at a time; do not fill
the entire interface with silent defaults.

## Your challenge

- [ ] Allocate each managed object from Rust-owned native memory using the
  simplest zero-initialized, correctly aligned strategy that satisfies the
  pinned object-header convention.
- [ ] Keep every returned object address stable for the rest of the process and
  enforce a small configured allocation limit so the intentional leak is
  bounded.
- [ ] Implement only the handle storage required by observed startup calls.
  Growing the store must not invalidate a handle address already given to
  CoreCLR.
- [ ] Supply a pinned-contract-valid no-collection write-barrier configuration.
  It may deliberately make card updates irrelevant only because this mission
  never collects; document that reasoning and fail before managed execution if
  the pinned runtime cannot represent it safely.
- [ ] Contain Rust panics and C++ exceptions at every boundary. Convert an
  allocation failure into one deterministic native failure path.
- [ ] Change the smoke command so this mission expects normal process completion
  rather than the deliberate initialization failure from mission 01.
- [ ] Keep the same sample runnable under the stock GC.

## Checkpoint

```powershell
dotnet run --project samples/LoaderSmoke/LoaderSmoke.csproj
python scripts/build.py smoke
```

Both runs print the sample's success line and exit with code zero. The custom run
also reports a bounded nonzero allocation count. It must not print
`GC initialization failed`.

## Allowed shortcuts

- One native allocation per managed object is allowed.
- A global lock is allowed.
- Allocation contexts, contiguous segments, heap walking, generations, and
  collection are not required.
- An intentionally inert card/ephemeral-bound configuration is allowed only when
  no collection can observe it and the pinned runtime contract accepts it. Mark
  it for mandatory replacement in mission 13.
- Process-lifetime interface and allocation metadata are allowed within the
  configured limit.

## Known debt

Objects are scattered across native allocations, memory only grows, the heap
cannot be walked from its bytes, and runtime features beyond the startup path may
be wrong or unsupported. Do not disguise these limitations.

## What this unlocks

Mission 04 can drive the working prototype beyond `Hello, World!` and collect
evidence about which limitations deserve a model or a major allocator rewrite.

## Hints

Prefer a slow path whose invariants fit on one page. Log ranges and sizes, never
managed object contents. The book's ZeroGC is a behavioral reference, not a
final architecture.
