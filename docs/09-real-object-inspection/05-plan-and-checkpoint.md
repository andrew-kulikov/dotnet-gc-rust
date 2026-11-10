# 5. Plan and checkpoint

[Overview](README.md) · [Previous: suspended inspection](04-suspended-inspection.md)

The six bounded steps were completed in order. The records below distinguish
source-derived facts, synthetic evidence, and runtime evidence.

1. **Supported shape — complete.** Check the source pin and narrow Windows x64
   layout. Add a bounded synthetic example for size 24, adjustment -16, count 1,
   and slot `[8, 16)`. Do not dereference managed memory.
2. **Object bounds and metadata witness — complete.** Add only the necessary read-only ledger
   snapshot with fallible bounded allocation, released before suspension. Check
   exact starts, prefix and full ranges with checked arithmetic, and establish
   the trusted `Node` witness and its lifetime. Test null, interior, misaligned,
   truncated, oversized and overflowing ranges; a mismatched MethodTable must
   cause no reads from the foreign address.
3. **One-series decoder — complete.** Use bounded bytes or a small callback, not a new
   adapter framework. Separate layout/size validation, slot decoding, and
   target reading. Test signed adjustment and malformed or truncated metadata
   with a fixed amount of work, including extreme counts and offsets.
4. **Live inspection — complete.** Prepare handles, scalar IDs, witness and independent
   expectations before P/Invoke. Inspect one source in the existing suspension
   callback, resolve target identity against handles, and return only logical
   observations. Verify `A -> B` and `A -> null`, size 24, one slot at offset 8,
   and managed continuation. Do not trace the graph.
5. **Failures and repeat cycles — complete.** Exercise decoder errors and mismatch through
   lifecycle callbacks, including restart failure priority and nested admission.
   Inject safe operation errors at runtime, repeat inspection with a bounded
   worker, and change `Next` between cycles. Verify one restart attempt and
   continued progress. Test corrupt inputs only on bounded synthetic data.
6. **Checkpoint and review — complete.** Add a focused runner with visible output, exit
   checks, a managed-process timeout, and only checkpoint assertions. Keep logs
   under `out/`. Run focused tests, Miri on runtime-independent buffers, the new
   runtime mode, the existing regression checkpoint, and `python scripts/build.py verify`
   after Rust/FFI/C# changes. Review against
   [the mission review guide](../../tasks/REVIEW_GUIDE.md), record results and
   limitations, then update the mission checklist.

## Required coverage

The live test must compare a reference and null against independent fixture
expectations, verify changing `A.Next` from `B` to null changes the observation, and deliberately
fail an expectation while preserving restart and continuation. Synthetic tests
must cover truncated descriptor words; zero, negative, minimum signed and huge
counts; valid -16 and invalid lengths; invalid, misaligned and overflowing offsets;
null, interior, foreign and truncated source ranges; foreign MethodTable without
reading it; invalid target, unknown label, and snapshot budget exhaustion.
Lifecycle tests must cover operation error, restart failure priority and no retry,
and rejected reentry. A null target is a successful observation, not an error.

## Step 1 record

Changed: checked the pinned source definitions and added a synthetic 24-byte
descriptor example in `object.rs`. It constructs the three x64 little-endian
words and checks one 8-byte slot at offset 8 and the object boundary. It does
not access a managed object or implement a general decoder.

Baseline, 2026-11-02: `python scripts/graph_fixtures.py --fixture chain --suspend-repeat`
exited 0; stock and ZeroGC fixtures passed, including eight
suspension cycles, the injected operation failure, worker progress and managed
continuation. Logs are under `out/graph-fixtures/`. `cargo test --locked -p gc-rust --lib`
passed 42 tests before the change. The source checkout reports
`79d0c463f1b55624c874a11585f7e47731e8d675`.

After the change, `cargo test --locked -p gc-rust --lib synthetic_node_descriptor_describes_one_reference_slot`
passed one test. Formatting, `git diff --check`, and local documentation links
and code fences also passed. This is
synthetic evidence only: no runtime object, MethodTable, or GCDesc was read.
Next action: step 2, the bounded storage and trusted metadata contract.

## Step 2 record

Added a fallible ledger snapshot capped at 262,144 ranges, checked exact object
starts and extents, and gated a metadata-read callback on an untagged matching
witness. Focused `cargo test --locked -p gc-rust --lib inspection::tests`
passed two tests; the `range_snapshot_respects_budget_and_is_independent` test
passed separately. At this point the snapshot was not connected and Rust reported
it as unused; the fixture still needed to supply the live witness. These were
buffer/range tests only. Next: decode one series.

## Step 3 record

Added a bounded slice decoder for the supported flags, size and one positive
series. `cargo test --locked -p gc-rust --lib inspection::tests` passed four
tests, including truncated words, signed -16, extreme counts, invalid lengths,
offsets, flags and sizes. The decoder performs no live reads; unused-code warnings
remain until the next step connects it. Next: inspect real fixture objects.

## Step 4 record

Connected the decoder to the existing suspension owner with two owned normal
handles and a retained `typeof(Node).TypeHandle.Value` witness. A direct debug
run of `GraphFixtures ... --inspect` under the built ZeroGC exited 0 and read
`B`, then null, each with size 24, one slot at offset 8, one restart and managed
continuation. Raw output is in `out/graph-fixtures/chain.zerogc.inspect-step4.*`.
This confirms the selected fixture on the pinned local runtime; failure and
repeated-worker coverage remain. Next: exercise errors and repeat cycles.

## Step 5 record

Synthetic initialized buffers now cover reference, null, mismatch, unknown and
invalid targets, and a foreign MethodTable. The lifecycle test sends a corrupt
descriptor through the owner: operation failure attempts restart once, restart
failure takes priority, and nested admission is rejected. Focused Rust tests
passed; both inspection tests and the lifecycle integration test passed under
`cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib` with their
respective test filters. Miri first exposed a synthetic pointer derived from a
single field; deriving it from the whole test object fixed the failing test.
The nightly emitted an existing `fetch_update` deprecation warning elsewhere.

The direct `--inspect` run exited 0 for eight cycles with worker progress before
and after each, alternating B/null. Cycle 4 intentionally mismatched the expected
ID and returned an operation failure with one restart. The worker stopped,
managed handles were freed and managed execution continued. Logs are under
`out/graph-fixtures/chain.zerogc.inspect-step5.*`. This does not simulate a
runtime restart failure or physical memory corruption. Next: the named runner,
full checks and review.

## Step 6 record and review

Added `python scripts/graph_fixtures.py --fixture chain --inspect` and the same
command to the Windows graph-fixture workflow. The runner prints process output,
checks exits and checkpoint lines, applies the existing 30-second managed-process
timeout, and saves separate stdout/stderr under `out/graph-fixtures/`.

On 2026-11-02 the focused command (with `--log-level error`) exited 0, observing
four references, four nulls, the intentional mismatch, eight restart attempts and worker
progress. `python scripts/graph_fixtures.py --suspend-repeat --log-level error`
passed the six prior fixture scenarios. `python scripts/build.py verify` passed
formatting, Clippy with warnings denied, 50 Rust unit tests, doc tests and the
stock-GC smoke test. The focused Miri commands and their warning are recorded in
step 5. The managed fixture built with zero warnings and errors.

Self-review against the [review guide](../../tasks/REVIEW_GUIDE.md): **PASS WITH
DOCUMENTED DEBT**. No blocking finding remains. The live results confirm the
source-derived size and slot for this fixture; they do not validate other types
or runtimes. Synthetic tests validate bounded rejection, not recovery from a
corrupted loader heap, invalid handle, native fault, or a failed real restart.
The reader accepts one candidate label, uses a capped ZeroGC allocation snapshot,
and does not establish reachability or authorize reclamation. Existing ZeroGC
normal-handle destruction is a no-op and retains the native handle records even
after `GCHandle.Free`. Next action:
Mission 10 may trace supported links from explicit fixture roots.

## Readability follow-up, 2026-11-09

Baseline: `python scripts/graph_fixtures.py --fixture chain --inspect` exited 0.
Moved the suspension and inspection exercises to `Diagnostics.cs`, named their
interop statuses, and print the returned observations. The Python runner now
uses each program's exit status rather than parsing output; the Rust model checks
its own expected live set. The metadata witness check now sits directly before
the read, with the foreign-table case exercised on a bounded test object.

Afterward, the focused inspection command with `--log-level error` and
`python scripts/graph_fixtures.py --log-level error --fixture chain --suspend-repeat`
exited 0. The inspection
observed reference, null, and the intentional mismatch with one restart per
cycle. `python scripts/build.py verify` passed, including 49 Rust unit tests;
`cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib inspection::tests`
passed four tests with an existing unrelated deprecation warning. Logs remain
under `out/graph-fixtures/`.

Limitations: the runner no longer independently checks printed edges, roots,
or which GC loaded; an incorrect graph with the same reachable set can pass.
The metadata and runtime scope above remain unchanged. Next: review these study
examples before beginning Mission 10.

## Cycle readability follow-up, 2026-11-09

The focused inspection baseline exited 0. Both diagnostic modes now share one
small eight-cycle worker loop, with progress checked before and after each call;
the worker synchronization stays in `BoundedMutator`. Removed per-cycle worker
messages, redundant object `KeepAlive` calls, and checks of the local handle
flags immediately after `Free`. Handles are still freed in `finally` and the
type witness stays alive through the call.

`python scripts/graph_fixtures.py --log-level error --fixture chain --inspect`
and the same command with `--suspend-repeat` exited 0 after the change; the
inspection still observed reference, null, and the intentional mismatch.
`python scripts/build.py verify` passed, including 49 Rust unit tests. One
concurrent baseline build failed on a shared output-file lock; the sequential
runs succeeded. Logs are under `out/graph-fixtures/`.

The normal handles remain specific to this diagnostic handoff; ZeroGC still
does not destroy their native records. This exercise does not test moving GC
or general handle ownership. Next: continue the review of task 9 before task 10.

## Naming and explicit cases, 2026-11-09

The focused `--inspect` baseline exited 0. Renamed the inspected node, known
next node, trusted Node MethodTable, configured/expected next-node IDs, and
observation field across C# and Rust. The inspection and repeated-suspension
examples now list their eight calls explicitly, including the deliberate error;
worker progress and restart checks remain in place. The interop layout is unchanged.

After the changes, `python scripts/graph_fixtures.py --log-level error --fixture chain --inspect`
and the corresponding `--suspend-repeat` run exited 0. The
inspection again observed reference, null and the intentional mismatch.
`python scripts/build.py verify` passed after the Rust and interop renames; the
managed sample was rebuilt with warnings as errors after the explicit calls.
Logs are under `out/graph-fixtures/`. The supported runtime and object shape are
unchanged; next, review this checkpoint before proceeding to mission 10.

## Inspection readability follow-up, 2026-11-09

The focused `--inspect` baseline exited 0. Combined the inspection setup and
result checks in the local `Inspect` function, which captures its handles and
MethodTable rather than passing them through a separate five-argument helper.
Success messages now show the observed ID, status, size and slot; errors also
include lifecycle, restart and slot-count details. All previous result checks
remain.

Afterward, `python scripts/graph_fixtures.py --log-level error --fixture chain --inspect`
exited 0 with reference, null and the intentional mismatch; the managed build
had zero warnings and errors. Logs are under `out/graph-fixtures/`. The fixture
still supports only the pinned Node shape. If a cycle throws, the worker is
asked to stop but the current `RunWithWorker` skips `Join`; review that failure
path before extending these diagnostics. Next: continue the task 9 review.

## Worker cleanup, 2026-11-09

The `--suspend-repeat` baseline exited 0. Moved the bounded worker `Join` into
`finally`, so it also runs when a managed diagnostic callback throws. The same
`python scripts/graph_fixtures.py --log-level error --fixture chain --suspend-repeat`
command then exited 0, including the injected native operation failure.

A separate stock-GC probe under `out/worker-cleanup-probe/` used a temporary
instrumented copy of the worker, delayed its exit, and threw from the managed
callback. `dotnet run --project out/worker-cleanup-probe/GraphFixtures.csproj -warnaserror`
exited 0 after confirming the original exception reached the caller only after
the worker exited. This verifies the managed cleanup path, not a runtime
restart failure. A join timeout raises its own exception. Next: continue the
review of task 9.

## Typed object addresses, 2026-11-09

The focused inspection and Rust inspection tests passed before the change.
`inspect_node` now receives the existing opaque `Object` type from the resolved
handles and reads the reference slot into the same type. Conversion to a raw
byte pointer stays at the memory-reading boundary. `Object` does not prove
liveness, initialization, bounds or suspension; the same unsafe preconditions
and checks still apply. No interop layout or supported object shape changed.

Afterward, `python scripts/graph_fixtures.py --log-level error --fixture chain --inspect`
exited 0, and `python scripts/build.py verify` passed with 49 unit tests.
`cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib inspection::tests`
passed four synthetic tests with the existing unrelated deprecation warning.
Miri does not validate actual CoreCLR memory. Runtime logs remain under
`out/graph-fixtures/`. Next: review task 9 before starting task 10.
