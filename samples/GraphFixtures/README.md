# Shared managed graph fixtures (Mission 07)

From the repository root, run the full Windows x64 checkpoint:

```sh
python scripts/graph_fixtures.py
```

This checks the Rust model, builds the managed sample and existing ZeroGC shim,
then runs each scenario in a fresh process under stock workstation GC and ZeroGC.
Every supported run must export the exact graph, roots, and expected live labels
and reach its continuation marker. Logs go to `out/graph-fixtures/`.
The `Graph fixtures` Windows workflow runs the same command on pull requests.
On other systems explicitly select the partial checkpoint:

```sh
python scripts/graph_fixtures.py --stock-only
```

## Independent expectation

`scenarios.txt` is the hand-written oracle. Each non-comment line contains four
whitespace-separated fields: scenario name, comma-separated `label:next` pairs,
explicit root labels (duplicates allowed), and sorted expected live labels.
A dash means a null `Next`, no roots, or an empty live set. Labels are sidecar
metadata independent of addresses. The runner bounds the input to 16 scenarios,
16 nodes per scenario, and 16 roots before managed construction.

| Scenario | Links | Explicit roots | Expected live labels |
| --- | --- | --- | --- |
| chain | A -> B -> C -> null | A | A, B, C |
| cycle | A -> B -> C -> A | A | A, B, C |
| disconnected | A -> B -> null; C -> D -> C | A | A, B |
| duplicate-roots | A -> B -> null | A, A | A, B |
| null-fields | A -> null; B -> null | A | A |
| no-roots | A -> B -> A | none | none |

C# constructs non-finalizable `Node` objects with exactly one reference field.
At the no-inline `Observe` control point it exports edges read from actual
`Next` fields, walks those references using identity, and checks the manually
declared live set. The runner checks exported edges and roots too: an incorrect
but equally reachable graph cannot pass. Rust builds the same logical graph
using `Graph`, checks marking against the oracle, then sweeps.
No collector implementation or external parsing package is added.

Ordered markers: `constructed`, `observation declared-only
raw-handoff=unsupported`, the `graph` record, and `continuation`.
The observation marker is a capability statement, not a native operation.
Arrays remain rooted through `GC.KeepAlive` after observation and continuation.
This deliberately retains disconnected nodes: fixture roots are a chosen input
to graph reachability, not an inventory of CoreCLR roots. An empty explicit
root set does not assert that stock GC has collected those objects.

## Pinned-runtime handoff contract

Target: Windows x64 workstation CoreCLR 10.0.11, SDK 10.0.400, and
`external/dotnet-runtime` at `79d0c463f1b55624c874a11585f7e47731e8d675`.
This describes the future diagnostic handoff. Mission 07 implements **no
raw-reference handoff or native memory read**. Both collectors support declared
managed observation here; both raw-handoff paths are explicitly unsupported.

1. Keep objects and explicit roots strongly reachable for the entire synchronous
   diagnostic call. `GC.KeepAlive` extends lifetime, not address stability;
   it does not freeze `Next` or stop another thread.
2. For the next mission, use owned normal `GCHandle` roots and pass only their
   `GCHandle.ToIntPtr` tokens through an ordinary GC-transitioning P/Invoke.
   A token is an opaque handle, **not an object address**. Allocate handles
   before native entry and release them in managed `finally` after return.
   Do not use `SuppressGCTransition` on a call requesting suspension.
3. Resolve handles only after successful suspension on a permitted thread,
   using verified contracts from this exact runtime. The pinned
   [`gchandleutilities.h`](../../external/dotnet-runtime/src/coreclr/vm/gchandleutilities.h)
   contains `ObjectFromHandle`: an internal helper, not an exported API the
   current shim can call. No guessed cast/dereference is permitted. Mission 08
   must prove transition, resolution, handle support, and exactly-once restart
   before this path becomes supported.
4. Borrow resolved references only inside that uninterrupted stopped window.
   Do not allocate managed objects, call back into managed code, unwind across
   FFI, or retain dereferenceable object pointers after restart. Return copied
   labels/counts/status. A normal handle keeps an object alive across return
   but does not preserve its address across subsequent GCs.
5. Do not pin `Node` and call `AddrOfPinnedObject`. The pinned
   [`GCHandle.cs`](../../external/dotnet-runtime/src/libraries/System.Private.CoreLib/src/System/Runtime/InteropServices/GCHandle.cs)
   rejects values for which `Marshal.IsPinnable` is false; `Node` contains a
   managed reference. Do not derive addresses with `Unsafe`, `TypedReference`,
   casts of managed values, or serialized pointer integers.

`Observe` is a managed control point for that later handoff, not a native ABI.
Mission 08 owns the observation window; missions 09-10 establish native decoding
and tracing. Declared edges alone cannot prove native object reading.
