# 2. The first empty cycle

[Mission overview](README.md) · [Previous: the suspension contract](01-observe-first.md)

This chapter records the first-cycle milestone. The remaining work listed below
was subsequently completed in [chapter 3](03-repeated-cycles.md) and
[chapter 4](04-handoff-checkpoint.md).

ZeroGC supports one synchronous empty diagnostic cycle from the managed fixture.
The lifecycle owner, C++ bridge, and required GC callbacks form the first
implemented part of Mission 08. This chapter describes that implementation and
what its checks establish.

## Lifecycle ownership

[`suspend.rs`](../../crates/gc-rust/src/suspend.rs) owns this sequence:

```text
Running → Suspending → Suspended → Restarting → Running
```

An atomic admission check accepts only `Running`. Nested or concurrent requests
are rejected immediately. `Lifecycle::run` takes three `FnOnce` callbacks:
suspend, diagnostic operation, and restart. Tests replace these calls with
closures to inject failures without loading CoreCLR.

After successful suspension, a private stack-bound guard owns restart. It
consumes the restart callback before invoking it, so an error cannot cause a
second attempt from `Drop`. An operation error or a Rust unwind through the
guard also triggers one restart attempt. A successful restart restores
`Running`; an injected suspend or restart error leaves terminal `Failed` state.

Both runtime calls remain on the same thread and call stack; the guard cannot
be handed off to another thread.

The guard cannot recover a process abort, panic-abort, hung runtime call, or
partially failed CoreCLR call. An unexpected panic at the extern-C boundary
terminates the process instead of unwinding into C++.

## The native entry and result

C# calls the diagnostic entry through ordinary `LibraryImport`. The C++ shim
forwards calls between Rust and CoreCLR using the existing callback table.
The native entry invokes `SUSPEND_FOR_GC_PREP` and `RestartEE(false)`.

The stopped operation is empty. It performs no logging, sleeping, allocation,
or managed callbacks. The owner holds no allocator, handle, or logging mutex
across the runtime pair.

The entry returns an 8-byte copied result containing a status and a restart
attempt count. It contains no managed references.

| Status | Meaning |
| --- | --- |
| 0 | Suspend, operation, and restart returned successfully |
| 1 | Entry rejected because the lifecycle was not `Running` |
| 2 | Injected suspend failure |
| 3 | Operation failure after a successful restart |
| 4 | Injected restart failure; takes priority over an operation failure |

CoreCLR's suspend/restart methods return `void`. Statuses 2 and 4 are exercised
through injected callbacks, not runtime error returns.

See [shim.cpp](../../native-shim/src/shim.cpp),
[rust_gc.h](../../native-shim/include/rust_gc.h), and
[Program.cs](../../samples/GraphFixtures/Program.cs) for the bridge and caller.

## State and completion callbacks

[`suspension_state.rs`](../../crates/gc-rust/src/suspension_state.rs) maintains
the EE's progress flag, a balanced suspension-pending counter, and a manual-reset
completion condition built from `Mutex<bool>` and `Condvar`.

The event mutex is held only within individual callbacks. Waiting releases it;
signaling wakes all waiters. A signal before a wait is remembered, and spurious
wakes recheck the condition. The allocator does not acquire this mutex.

In the pinned `gcee.cpp`, `bConsiderGCStart` additionally considers `gc_started`,
not suspension pending. ZeroGC starts no collections, so both argument values
query the same EE progress flag and use the same completion wait. `GetGcCount`
remains zero.

Native tests cover two waiting threads and a signal that precedes a wait. They
do not establish CoreCLR's managed waiting behavior during repeated cycles.

### Allocation-context cleanup

[`rust_gc_fix_alloc_context`](../../crates/gc-rust/src/gc_heap.rs) implements the
`void FixAllocContext` callback for ZeroGC's slow allocator. The allocator returns
separately allocated storage and never supplies a fast-path allocation area.
A live context therefore has null allocation pointers and null GC-reserved
fields; its allocation counters may be nonzero.

The callback validates those fields and preserves the counters. A null context
pointer or non-null allocation/reserved field terminates the process at the
FFI boundary. An active allocation tail is unsupported and is never silently
discarded.

This matches the current allocator and thread-exit path:
`Thread::CooperativeCleanup` passes its live context with null `arg` and `heap`,
then reinitializes it. The stock collector's active-context retirement work is
outside this implementation. Allocation-context enumeration and heap-association
queries remain unsupported.

## Runtime validation

Run the focused probe from the repository root:

```powershell
python scripts/graph_fixtures.py --fixture chain --suspend
```

The recorded run exited 0. It checked `chain` in the Rust model and under stock
GC, then completed one empty diagnostic cycle under ZeroGC with these records:

```text
dotnet: Managed execution was restarted once.
dotnet: Graph 'chain':
dotnet: Edges: A:B,B:C,C:-
dotnet: Roots: A
dotnet: Reachable nodes: A,B,C
dotnet: Managed execution continued after 'chain'.
```

The suspension record is printed after native return. Together with status 0,
it establishes that suspend returned, the empty operation completed, and one
restart attempt returned. The continuation record confirms later managed
execution. The raw logs are
`out/graph-fixtures/chain.zerogc.suspend.{stdout,stderr}.log`.

The [runner](../../scripts/graph_fixtures.py) displays process output, checks
exit codes, graph/marker order, and GC initialization, and applies a 30-second
timeout to managed scenarios. A timeout is a failed experiment; restart must
not be attempted from another thread to recover the process.

### Regression checks

These are recorded results from implementation validation, using the pins in
chapter 1:

| Command | Observed result |
| --- | --- |
| `python scripts/graph_fixtures.py` | All six model scenarios and twelve stock-GC/ZeroGC processes passed |
| `cargo test -p gc-rust --lib` | 39 tests passed, including five lifecycle tests, completion-event tests, and allocation-context preservation |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo fmt --all -- --check` | Exit 0 |
| `python scripts/build.py matrix --report-dir out/fix-alloc-context-review/matrix` | Exit 0, including `threads`, named unsupported scenarios, and allocation exhaustion |

The bounded `LoaderSmoke threads` scenario also exited 0 under both stock
workstation GC and debug ZeroGC. ZeroGC logged two `rust_gc_fix_alloc_context()`
calls before `scenario threads ok`; raw logs are under
`out/fix-alloc-context-review/`. This establishes the existing thread-exit case.

For focused lifecycle checks:

```powershell
cargo test -p gc-rust --lib suspend::tests
cargo test -p gc-rust --lib suspend::tests::operation_error_and_panic_each_restart_once -- --exact
```

## Next: a second managed mutator

The next experiment repeats the empty diagnostic cycle with one additional
bounded managed worker. It must show worker progress before suspension and
after restart, managed continuation after successful cycles, and clean worker
exit. Counter observations need explicit synchronization. Two equal samples
alone do not prove suspension: the worker may simply not have run.

The current probe has one application mutator. Native failure injection,
native waiter tests, and a separate thread-exit scenario do not establish
repeated CoreCLR suspension with two active mutators. Handle handoff and
borrowed-reference lifetime also remain unvalidated. No object reading,
tracing, or collection is performed by this cycle.
