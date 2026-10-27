# 4. Handle handoff and the completed checkpoint

[Mission overview](README.md) · [Previous: repeated cycles](03-repeated-cycles.md)

Mission 08 passes on the pinned Windows x64 workstation CoreCLR 10.0.11 runtime.
The baseline empty cycle was reproduced before changes. The completed probe adds
one owned normal handle, repeated suspension with a second bounded mutator, and
an operation failure inside the stopped window. It performs no object decoding,
heap walk, root scan, managed allocation, or collection in that window.

## Verified handle contract

These are facts from the pinned source, not an inferred object layout:

- [`GCHandle.cs`](../../external/dotnet-runtime/src/libraries/System.Private.CoreLib/src/System/Runtime/InteropServices/GCHandle.cs)
  returns the internal token from `ToIntPtr`; only pinned handles receive a low-bit
  tag. Normal handles use type 2, matching `HNDTYPE_STRONG` in
  [`gcinterface.h`](../../external/dotnet-runtime/src/coreclr/gc/gcinterface.h).
- [`GCHandle.CoreCLR.cs`](../../external/dotnet-runtime/src/coreclr/System.Private.CoreLib/src/System/Runtime/InteropServices/GCHandle.CoreCLR.cs)
  and [`marshalnative.cpp`](../../external/dotnet-runtime/src/coreclr/vm/marshalnative.cpp)
  route allocation to `IGCHandleStore::CreateHandleOfType` and release to
  `IGCHandleManager::DestroyHandleOfUnknownType` for this workload.
- The existing [shim](../../native-shim/src/shim.cpp) returns the address of
  ZeroGC's [stable handle record](../../crates/gc-rust/src/handle_store.rs).
  Its first field is the object slot. The internal `ObjectFromHandle` helper is
  not an exported runtime API and is not called by the probe.

C# allocates `GCHandleType.Normal` before native entry and owns it exclusively
until `finally` calls `Free`. An ordinary GC-transitioning P/Invoke passes the
opaque token. After `SuspendEE` returns, `check_diagnostic_handle` checks the
record's strong-handle type and non-null object slot. Only `Result<(), ()>` leaves
that scope; no borrowed object address reaches restart or the returned ABI record.
No managed object bytes are read. Managed code checks target identity before
entry and after return. No registry, handle algorithm, or public resolver was added.

The entry accepts only live handles from this ZeroGC instance. Its unsafe
preconditions exclude foreign, freed, fabricated, or concurrently changed tokens;
null/misaligned tokens, unsupported stored kinds, and null targets fail the
operation. Unit tests check supported normal targets, null handles/targets, and
weak/pinned records. This is not a validator for arbitrary pointer values.

## Failure and continuation evidence

`--suspend-repeat` first runs a successful handle cycle with one application
mutator, then eight cycles with the second mutator. Cycle 4 requests an error
after successful handle resolution and before restart. It must return status 3
and one restart attempt. Every other cycle must return status 0 and one attempt.
The worker must make synchronized progress before and after every repeated
cycle, then exit cleanly. The managed handle is released, collection count stays
zero, the graph matches its oracle, and the continuation marker must follow.

Output contains observed results: the handle check, restart count, worker
progress/exit, handle release, the graph, and managed continuation. Static
capability notices and generic ready/completed messages are omitted. The graph
is checked in managed code; native object reading remains unsupported, as
documented here rather than printed on each run. The runner checks result
sentences in order, for example:

```text
dotnet: The handle was checked and still points to the expected object.
dotnet: Managed execution was restarted once.
...
dotnet: Worker made progress before cycle 4.
dotnet: Cycle 4 reported the expected operation failure.
dotnet: Managed execution was restarted once.
dotnet: Worker made progress after cycle 4.
...
dotnet: Worker stopped.
dotnet: Handle released.
```

The fixture uses interpolation for string construction and braces for every
control-flow block. [Dependent-handle support](05-dependent-handles.md) lets
numeric interpolation use the runtime's normal formatting path.

After removing the capability notice, ready/completed boilerplate, and graph-header
commentary, `python scripts/graph_fixtures.py --suspend-repeat` exited 0 for all
six fixtures. `python scripts/graph_fixtures.py --fixture chain --suspend` also
exited 0. Result assertions remain in place. Each run built the managed fixture
with warnings treated as errors. `git diff --check` passed. Runtime semantics
and the limitations below are unchanged; the next action remains Mission 09.

The runtime probe passed all six fixtures: six single-mutator handoff cycles and
48 cycles with two mutators, including six injected operation failures. Every
cycle reported one restart attempt. All six workers exited and all continuation
markers appeared. Logs are saved under
`out/graph-fixtures/<fixture>.zerogc.suspend-repeat.{stdout,stderr}.log`.

Native lifecycle tests independently cover nested entry and every non-running
state, failed suspension without restart, operation error and Rust unwind with
one restart, and restart failure after either successful or failed operation.
Restart failure wins over operation failure, leaves terminal `Failed`, and
rejects the next request without retrying restart.

CoreCLR's suspend/restart methods return `void`: restart failure is injected only
through native test callbacks. The actual runtime fault injection fails the
diagnostic operation, then calls the real restart. It does not simulate a
recoverable EE error, partial suspension, abort, or hung runtime call. Unexpected
panic cannot unwind across the extern-C boundary.

## Reproduction and observed results

Run the full checkpoint from the repository root:

```powershell
python scripts/build.py verify
python scripts/graph_fixtures.py
python scripts/graph_fixtures.py --suspend-repeat
```

Recorded results on 2026-10-27:

| Command | Observed result |
| --- | --- |
| `python scripts/build.py verify` | Exit 0: formatting, Clippy with warnings denied, 42 unit tests, six doctests, stock-GC smoke; intentional UB demonstration remains ignored |
| `python scripts/graph_fixtures.py` | Exit 0: six model scenarios and twelve baseline stock/ZeroGC processes |
| `python scripts/graph_fixtures.py --fixture chain --suspend` | Exit 0 before and after changes: original empty cycle, one restart attempt, managed continuation |
| `python scripts/graph_fixtures.py --suspend-repeat` | Exit 0: all six model/stock cases and all six ZeroGC handoff/repeated-cycle cases described above |
| `python scripts/build.py matrix --report-dir out/dependent-handles/matrix` | Exit 0: supported scenarios including dependent handles pass; `GetLOHThreshold` and `RegisterForFinalization` remain explicit unsupported failures; allocation exhaustion remains deterministic |
| `cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib handle_store::tests` | Four tests passed |
| `cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib suspend::tests` | Five tests passed |

The pinned Miri toolchain warns that the existing `fetch_update` call is deprecated;
both Miri runs exit 0. The pinned stable Clippy run reports no warnings. No
unrelated toolchain/API migration was made. The Windows graph workflow now runs
both the baseline and `--suspend-repeat` checkpoint after `verify`.

For the narrower probes use `--fixture chain --suspend` (original empty cycle)
or `--fixture chain --suspend-repeat` (handoff, repeated cycles, injected error).
Suspension options are mutually exclusive and require ZeroGC; `--stock-only`
cannot silently skip the requested suspension. Managed process timeout remains
30 seconds; no recovery restart is attempted from another thread.

## Limitations and next action

The fixture deliberately retains all nodes and passes just one normal handle to
its first node, including in the no-explicit-roots case. This establishes a
borrowed-reference handoff, not runtime root coverage, liveness, or collection.
`Free` releases the managed handle ownership; ZeroGC's existing destroy callback
keeps the normal-handle record until process exit; explicitly disposed dependent
records are freed. Collection-time handle tracing remains future work. No
diagnostic object pointer is retained outside the stopped operation.

Progress/continuation evidence and native completion-event tests do not assert
coverage of a particular CoreCLR waiting branch. No new callback was needed for
the second mutator: existing progress/event callbacks and `FixAllocContext`
sufficed, with worker-exit cleanup observed. Allocation fast paths and active
context retirement remain unsupported.

Self-review verdict: **PASS WITH DOCUMENTED DEBT** for Mission 08. Next action is
Mission 09: establish the supported object layout before reading one object's
fields inside this verified suspension window.
