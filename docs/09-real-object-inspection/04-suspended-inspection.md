# 4. Inspection during suspension

[Overview](README.md) · [Previous: GCDesc](03-gcdesc.md) · [Next: checkpoint](05-plan-and-checkpoint.md)

The fixture creates `nodeToInspect` (`A`) and `knownNextNode` (`B`) and retains
a normal handle for each. It also supplies `nodeMethodTable`: the trusted
non-collectible `Node` MethodTable obtained from `typeof(Node).TypeHandle.Value`
on the pinned runtime. Rust uses it to validate object types before reading
metadata. An ordinary GC-transitioning P/Invoke calls `rust_gc_inspect_node` in
[suspend.rs](../../crates/gc-rust/src/suspend.rs). Objects and type stay alive
through the call; `finally` frees the managed handles. ZeroGC's existing
normal-handle destruction is a no-op, so their native records remain allocated.
Foreign, fabricated, freed,
or concurrently modified handles are outside the entry's unsafe contract.

The native entry snapshots allocation ranges, then uses the existing lifecycle
owner. Its operation resolves the handles only after suspension. It validates the
inspected object's prefix, compares its MethodTable with `nodeMethodTable`, copies the fixed
metadata span, decodes the descriptor, validates the full object range and
reads the slot. For a non-null target it checks the target's exact allocation
start, extent and MethodTable before comparing its address to `knownNextNodeHandle`. Actual ID 1 means `B`; ID 0 means null. The independent expected ID is
used only for comparison. The code neither allocates nor logs in the stopped
operation and copies no object or metadata pointer into the result.

The direct C# to Rust ABI returns a 28-byte, four-byte-aligned record: an 8-byte
lifecycle result followed by five `u32` fields (inspection status, object size,
slot offset, slot count, target ID). The Rust and managed declarations check
the size and observation offset. The C++ GC shim does not handle this entry.
Failure results have zero observation fields, except `ExpectationMismatch`,
which retains the actual size, slot and target ID for diagnosis.

| Inspection status | Value | Meaning |
| --- | ---: | --- |
| `NotRun` | 0 | Lifecycle rejected entry or stopped before the operation |
| `Reference`, `Null` | 1, 2 | One slot was read and matched the expectation |
| `InvalidObject` | 3 | Source handle target or allocation range is invalid |
| `UnsupportedLayout` | 4 | Type witness, flags or descriptor form is unsupported |
| `InvalidMetadata` | 5 | Known type has inconsistent flags or size |
| `CorruptDescriptor` | 6 | Descriptor bytes or computed slot are invalid |
| `InvalidTarget`, `UnknownTarget` | 7, 8 | Target is invalid, or valid but not candidate `B` |
| `ExpectationMismatch` | 9 | Actual target differs from the independent expectation |
| `BudgetExceeded` | 10 | Range snapshot failed before suspension |

Every post-suspend result passes through the restart guard. An operation error
returns `OperationFailed` after a successful restart; a restart failure takes
priority and is not retried. Failed admission never runs the operation. A
pre-suspend snapshot failure reports `BudgetExceeded` and zero restart attempts.
CoreCLR's suspend/restart callbacks return `void`; injected callbacks test
failure ordering, but aborts, faults and stuck runtime calls are not recoverable
`Result` values. Unexpected Rust panics cannot unwind across FFI.

The runtime checkpoint alternates `A.Next` between `B` and null over eight
cycles. Cycle 4 deliberately expects `B` while the actual slot is null; it
checks operation failure, the actual observation and one restart. A separate
bounded worker must advance before and after every cycle and stop before handle
release. Malformed metadata and targets are tested with owned buffers, not by
writing invalid bytes into the running managed heap.
