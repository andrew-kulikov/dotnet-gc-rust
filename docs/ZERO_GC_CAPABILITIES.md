# ZeroGC capability observations

Run the pinned Windows x64 scenario matrix with `python scripts/build.py matrix`.
It builds the debug native shim (with native call stacks), builds `LoaderSmoke`,
and runs each scenario in a fresh process under stock GC and ZeroGC. The matrix
checks the named outcome and counters. The sample also accepts one scenario name
as its sole command-line argument; no argument preserves `Hello, World!`.

To inspect the native reports, run:

```powershell
python scripts/build.py matrix --report-dir out/matrix-reports
Get-Content out/matrix-reports/exhaust.zerogc.1.stderr.log
```

The option writes separate `.stdout.log` and `.stderr.log` files for each
scenario and collector. The two exhaustion attempts have `.1` and `.2`
suffixes. `out/` is ignored by Git; use another directory if you want to keep
the reports elsewhere.

| Scenario | Stock GC | ZeroGC | Last collector method / reason |
| --- | --- | --- | --- |
| Minimal smoke | works | works | `IGCHeap::Shutdown` reports counters |
| Small-object churn | works | works | 512 arrays of 128 bytes |
| One large array | works | unsupported by named method | `IGCHeap::GetLOHThreshold` |
| Two allocating threads | works | unsupported by named method | `IGCHeap::FixAllocContext` |
| Pinned buffer | works | works | Pinned `GCHandle` exposes a non-null address |
| Finalization registration | works | unsupported by named method | `IGCHeap::RegisterForFinalization` via `GC.ReRegisterForFinalize` |
| Bounded exhaustion | works | deterministic exhaustion | `IGCHeap::Alloc` rejects the request at `ZERO_GC_LIMIT_BYTES=1048576` |

There are no unexpected defects in this matrix. An unsupported result is a
native abort with the method name, not a successful stub. The two-thread case
can print `FixAllocContext` twice because both threads may enter it before the
first abort terminates the process. The matrix checks the last named call.

`ZERO_GC_LIMIT_BYTES` is a positive decimal byte limit for managed-object
storage; omitted means 64 MiB. A request is counted when `IGCHeap::Alloc` enters.
`requested_bytes` sums raw request sizes, including a rejected request, and
saturates instead of wrapping if the counter overflows.
`successful` and `owned_bytes` count only live native object blocks, with
`owned_bytes` using each block's aligned allocation size. ZeroGC never frees
these blocks. `registry_bytes` is the current capacity of the diagnostic range
vector, separate from the object-storage limit. The vector is temporary
diagnostic metadata and must not become a liveness oracle.

The exit or abort report checks all registered native ranges for overlap and
prints the first two ranges plus the number of gaps between sorted ranges.
This shows that object storage is scattered across disjoint native allocations.
A byte-wise forward walk cannot advance from one object to the next through a
single heap range. ZeroGC reports this limitation explicitly; a later mission
must introduce an actual heap before collection can safely walk objects.
